use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use fret_core::{AppWindowId, NodeId, PointerId};
use fret_runtime::{ClipboardToken, FrameId, ImageUploadToken, TickId, TimerToken};

use crate::drag::{DragKindId, DragSession, DragSessionId};
use fret_runtime::{
    BindingV1, CommandRegistry, Effect, KeySpecV1, Keymap, KeymapFileV1, KeymapService, ModelHost,
    ModelId, ModelStore, WindowCommandEnabledService,
};

use crate::SettingsFileV1;
use crate::menu_bar::MenuBarBaselineService;

#[derive(Debug)]
struct GlobalLeaseMarker {
    type_name: &'static str,
    leased_at: &'static std::panic::Location<'static>,
}

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    global_type_names: HashMap<TypeId, &'static str>,
    #[cfg(debug_assertions)]
    global_last_changed_at: HashMap<TypeId, &'static std::panic::Location<'static>>,
    global_revisions: HashMap<TypeId, u64>,
    changed_globals: Vec<TypeId>,
    changed_globals_dedup: HashSet<TypeId>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw_requests: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drags: HashMap<PointerId, DragSession>,
    next_drag_session_id: u64,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
    next_clipboard_token: u64,
    next_image_upload_token: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            globals: HashMap::new(),
            global_type_names: HashMap::new(),
            #[cfg(debug_assertions)]
            global_last_changed_at: HashMap::new(),
            global_revisions: HashMap::new(),
            changed_globals: Vec::new(),
            changed_globals_dedup: HashSet::new(),
            models: ModelStore::default(),
            commands: CommandRegistry::default(),
            redraw_requests: HashSet::new(),
            effects: Vec::new(),
            drags: HashMap::new(),
            next_drag_session_id: 1,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            next_timer_token: 1,
            next_clipboard_token: 1,
            next_image_upload_token: 1,
        };

        // Provide a minimal default keymap so basic UI focus traversal works out of the box.
        // Apps can fully override this by installing their own `KeymapService`.
        app.set_global(default_keymap_service());

        app.set_global(fret_runtime::DockingInteractionSettings::default());
        app.set_global(fret_runtime::fret_i18n::I18nService::default());

        crate::core_commands::register_core_commands(app.commands_mut());
        crate::keymap::install_command_default_keybindings_into_keymap(&mut app);

        app
    }

    fn mark_global_changed(&mut self, id: TypeId) {
        if self.changed_globals_dedup.insert(id) {
            self.changed_globals.push(id);
        }
    }

    pub(crate) fn mark_global_changed_at(
        &mut self,
        id: TypeId,
        at: &'static std::panic::Location<'static>,
    ) {
        #[cfg(debug_assertions)]
        {
            self.global_last_changed_at.insert(id, at);
        }
        let _ = at;
        self.global_revisions
            .entry(id)
            .and_modify(|rev| *rev = rev.saturating_add(1))
            .or_insert(1);
        self.mark_global_changed(id);
    }

    pub fn take_changed_globals(&mut self) -> Vec<TypeId> {
        self.changed_globals_dedup.clear();
        std::mem::take(&mut self.changed_globals)
    }

    pub fn global_type_name(&self, id: TypeId) -> Option<&'static str> {
        self.global_type_names.get(&id).copied()
    }

    pub fn global_changed_at(&self, id: TypeId) -> Option<&'static std::panic::Location<'static>> {
        #[cfg(debug_assertions)]
        {
            self.global_last_changed_at.get(&id).copied()
        }

        #[cfg(not(debug_assertions))]
        {
            let _ = id;
            None
        }
    }

    pub fn global_revision(&self, id: TypeId) -> Option<u64> {
        self.global_revisions.get(&id).copied()
    }

    #[track_caller]
    fn assert_global_not_leased<T: Any>(value: &dyn Any) {
        if let Some(marker) = value.downcast_ref::<GlobalLeaseMarker>() {
            panic!(
                "global is currently leased: {} (type_id={:?}); leased at {}; accessed at {}",
                marker.type_name,
                TypeId::of::<T>(),
                marker.leased_at,
                std::panic::Location::caller()
            );
        }
    }

    #[track_caller]
    pub fn set_global<T: Any>(&mut self, value: T) {
        let changed_at = std::panic::Location::caller();
        self.set_global_at(value, changed_at);
    }

    pub(crate) fn set_global_at<T: Any>(
        &mut self,
        value: T,
        changed_at: &'static std::panic::Location<'static>,
    ) {
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.globals.get(&type_id) {
            Self::assert_global_not_leased::<T>(existing.as_ref());
        }
        self.globals.insert(type_id, Box::new(value));
        self.global_type_names
            .entry(type_id)
            .or_insert(std::any::type_name::<T>());
        self.mark_global_changed_at(type_id, changed_at);
    }

    #[track_caller]
    pub fn global<T: Any>(&self) -> Option<&T> {
        let value = self.globals.get(&TypeId::of::<T>())?;
        Self::assert_global_not_leased::<T>(value.as_ref());
        value.downcast_ref::<T>()
    }

    #[track_caller]
    pub fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
    ) -> R {
        let leased_at = std::panic::Location::caller();
        self.with_global_mut_impl(init, f, leased_at, true)
    }

    #[track_caller]
    pub fn with_global_mut_untracked<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
    ) -> R {
        let leased_at = std::panic::Location::caller();
        self.with_global_mut_impl(init, f, leased_at, false)
    }

    pub(crate) fn with_global_mut_impl<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
        leased_at: &'static std::panic::Location<'static>,
        mark_changed: bool,
    ) -> R {
        let type_id = TypeId::of::<T>();
        self.global_type_names
            .entry(type_id)
            .or_insert(std::any::type_name::<T>());
        let marker = GlobalLeaseMarker {
            type_name: std::any::type_name::<T>(),
            leased_at,
        };
        let existing = self.globals.insert(type_id, Box::new(marker));

        let mut value = match existing {
            None => init(),
            Some(v) => {
                if let Some(marker) = v.downcast_ref::<GlobalLeaseMarker>() {
                    panic!(
                        "global already leased: {} (type_id={type_id:?}); leased at {}; accessed at {}",
                        marker.type_name, marker.leased_at, leased_at
                    );
                }
                *v.downcast::<T>().expect("global type id must match")
            }
        };

        let result = if cfg!(panic = "unwind") {
            catch_unwind(AssertUnwindSafe(|| f(&mut value, self)))
        } else {
            Ok(f(&mut value, self))
        };

        let replaced = self.globals.insert(type_id, Box::new(value));
        let Some(replaced) = replaced else {
            panic!("global lease marker was removed unexpectedly: type_id={type_id:?}");
        };
        if !replaced.is::<GlobalLeaseMarker>() {
            panic!("global lease marker was replaced unexpectedly: type_id={type_id:?}");
        }

        if mark_changed {
            self.mark_global_changed_at(type_id, leased_at);
        }

        match result {
            Ok(value) => value,
            Err(panic) => resume_unwind(panic),
        }
    }

    pub fn models(&self) -> &ModelStore {
        &self.models
    }

    pub fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }

    pub fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    pub fn commands_mut(&mut self) -> &mut CommandRegistry {
        &mut self.commands
    }

    pub fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }

    pub fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new(session_id, pointer_id, source_window, kind, start, payload),
        );
    }

    pub fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        let session_id = DragSessionId(self.next_drag_session_id);
        self.next_drag_session_id = self.next_drag_session_id.saturating_add(1);
        self.drags.insert(
            pointer_id,
            DragSession::new_cross_window(
                session_id,
                pointer_id,
                source_window,
                kind,
                start,
                payload,
            ),
        );
    }

    pub fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        self.drags.get(&pointer_id)
    }

    pub(crate) fn drags(&self) -> impl Iterator<Item = &DragSession> {
        self.drags.values()
    }

    pub fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        self.drags.get_mut(&pointer_id)
    }

    pub fn end_drag(&mut self, pointer_id: PointerId) -> Option<DragSession> {
        self.drags.remove(&pointer_id)
    }

    pub fn cancel_drag(&mut self, pointer_id: PointerId) {
        self.drags.remove(&pointer_id);
    }

    /// Request a window redraw (one-shot).
    ///
    /// This records a pending redraw request which the runner will translate into an actual
    /// platform `request_redraw` call at a deterministic point in the event loop.
    ///
    /// Notes:
    /// - This is a one-shot request and may be coalesced by the runner or the platform.
    /// - This does not imply continuous frame progression. For animations or any behavior that
    ///   needs to advance without input events, use `Effect::RequestAnimationFrame` (typically via
    ///   `Cx::request_animation_frame()`), and re-issue it each frame while active.
    pub fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw_requests.insert(window);
    }

    /// Runner-owned monotonic tick id (increments once per event-loop turn).
    pub fn tick_id(&self) -> TickId {
        self.tick_id
    }

    /// Runner-owned monotonic frame id (increments on each render/present).
    pub fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    /// Runner-only.
    pub fn set_tick_id(&mut self, tick_id: TickId) {
        self.tick_id = tick_id;
    }

    /// Runner-only.
    pub fn set_frame_id(&mut self, frame_id: FrameId) {
        self.frame_id = frame_id;
    }

    pub fn next_timer_token(&mut self) -> TimerToken {
        let token = TimerToken(self.next_timer_token);
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        token
    }

    pub fn next_clipboard_token(&mut self) -> ClipboardToken {
        let token = ClipboardToken(self.next_clipboard_token);
        self.next_clipboard_token = self.next_clipboard_token.saturating_add(1);
        token
    }

    pub fn next_image_upload_token(&mut self) -> ImageUploadToken {
        let token = ImageUploadToken(self.next_image_upload_token);
        self.next_image_upload_token = self.next_image_upload_token.saturating_add(1);
        token
    }

    pub fn push_effect(&mut self, effect: Effect) {
        if let Effect::Command {
            window: Some(window),
            command,
        } = &effect
            && self
                .global::<WindowCommandEnabledService>()
                .is_some_and(|svc| svc.enabled(*window, command) == Some(false))
        {
            return;
        }

        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            Effect::SetMenuBar {
                window: None,
                ref menu_bar,
            } => {
                self.with_global_mut_untracked(MenuBarBaselineService::default, |svc, _app| {
                    svc.note_default_menu_bar(menu_bar);
                });
                self.effects.push(effect);
            }
            effect => self.effects.push(effect),
        }
    }

    pub fn flush_effects(&mut self) -> Vec<Effect> {
        let mut effects = std::mem::take(&mut self.effects);
        for window in self.redraw_requests.drain() {
            effects.push(Effect::Redraw(window));
        }

        let platform = fret_runtime::Platform::current();
        let os_menu_enabled = self
            .global::<SettingsFileV1>()
            .map(|s| s.menu_bar_os_enabled(platform))
            .unwrap_or_else(|| SettingsFileV1::default().menu_bar_os_enabled(platform));

        if !os_menu_enabled {
            effects.retain(|e| match e {
                Effect::SetMenuBar { menu_bar, .. } => menu_bar.menus.is_empty(),
                _ => true,
            });
        }
        effects
    }
}

#[cfg(test)]
mod menu_bar_effect_tests {
    use super::*;
    use fret_runtime::{Menu, MenuBar, MenuItem};

    #[test]
    fn flush_effects_filters_os_menubar_when_disabled_but_keeps_clear() {
        let mut app = App::new();

        let mut settings = SettingsFileV1::default();
        settings.menu_bar.os = crate::MenuBarIntegrationModeV1::Off;
        app.set_global(settings);

        app.push_effect(Effect::SetMenuBar {
            window: None,
            menu_bar: MenuBar {
                menus: vec![Menu {
                    title: "File".into(),
                    role: None,
                    mnemonic: None,
                    items: vec![MenuItem::Separator],
                }],
            },
        });

        app.push_effect(Effect::SetMenuBar {
            window: None,
            menu_bar: MenuBar::empty(),
        });

        let effects = app.flush_effects();
        assert!(
            effects.iter().any(
                |e| matches!(e, Effect::SetMenuBar { menu_bar, .. } if menu_bar.menus.is_empty())
            ),
            "empty SetMenuBar should be retained so runners can clear a previously-published OS menubar"
        );
        assert!(
            !effects.iter().any(
                |e| matches!(e, Effect::SetMenuBar { menu_bar, .. } if !menu_bar.menus.is_empty())
            ),
            "non-empty SetMenuBar should be filtered when OS menubar is disabled"
        );
    }
}

#[cfg(test)]
mod command_enabled_effect_tests {
    use super::*;
    use fret_runtime::{CommandId, WindowCommandEnabledService};

    #[test]
    fn push_effect_drops_window_scoped_command_when_disabled() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let cmd = CommandId::from("app.preferences");

        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        app.push_effect(Effect::Command {
            window: Some(window),
            command: cmd.clone(),
        });

        let effects = app.flush_effects();
        assert!(
            !effects.iter().any(|e| {
                matches!(
                    e,
                    Effect::Command { window: Some(w), command } if *w == window && command == &cmd
                )
            }),
            "disabled window-scoped commands should be dropped as a final guardrail"
        );
    }

    #[test]
    fn push_effect_keeps_global_command_even_if_a_window_override_disables_it() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let cmd = CommandId::from("app.preferences");

        app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
            svc.set_enabled(window, cmd.clone(), false);
        });

        app.push_effect(Effect::Command {
            window: None,
            command: cmd.clone(),
        });

        let effects = app.flush_effects();
        assert!(
            effects
                .iter()
                .any(|e| matches!(e, Effect::Command { window: None, command } if command == &cmd)),
            "window-scoped overrides should not affect global commands"
        );
    }
}

fn default_keymap_service() -> KeymapService {
    KeymapService {
        keymap: Keymap::from_v1(KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("focus.next".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("focus.previous".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["shift".into()],
                        key: "Tab".into(),
                    },
                },
                BindingV1 {
                    command: Some("toast.viewport.focus".into()),
                    platform: None,
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec![],
                        key: "F8".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("windows".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("linux".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("web".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("web".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("web".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("web".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyA".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.copy".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into()],
                        key: "KeyC".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.cut".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into()],
                        key: "KeyX".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.paste".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into()],
                        key: "KeyV".into(),
                    },
                },
                BindingV1 {
                    command: Some("text.select_all".into()),
                    platform: Some("macos".into()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into()],
                        key: "KeyA".into(),
                    },
                },
                // Document/window-level undo/redo (ADR 0136).
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("windows".into()),
                    when: Some("edit.can_undo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("windows".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyY".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("windows".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("linux".into()),
                    when: Some("edit.can_undo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("linux".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyY".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("linux".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("web".into()),
                    when: Some("edit.can_undo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("web".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into()],
                        key: "KeyY".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("web".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.undo".into()),
                    platform: Some("macos".into()),
                    when: Some("edit.can_undo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into()],
                        key: "KeyZ".into(),
                    },
                },
                BindingV1 {
                    command: Some("edit.redo".into()),
                    platform: Some("macos".into()),
                    when: Some("edit.can_redo".into()),
                    keys: KeySpecV1 {
                        mods: vec!["cmd".into(), "shift".into()],
                        key: "KeyZ".into(),
                    },
                },
            ],
        })
        .expect("default keymap must be valid"),
    }
}
impl ModelHost for App {
    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Focus {
    pub node: NodeId,
}

#[cfg(test)]
mod tests {
    use super::{App, default_keymap_service};
    use fret_core::{KeyCode, Modifiers};
    use fret_runtime::{
        CommandId, InputContext, InputDispatchPhase, KeyChord, Platform, fret_i18n,
    };
    use std::any::TypeId;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn with_global_mut_restores_value_on_panic() {
        let mut app = App::new();
        app.set_global::<u32>(1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            app.with_global_mut(
                || 0u32,
                |v, _app| {
                    *v = 2;
                    panic!("boom");
                },
            );
        }));
        assert!(result.is_err());

        assert_eq!(*app.global::<u32>().unwrap(), 2);
    }

    #[test]
    fn global_access_panics_while_leased() {
        let mut app = App::new();
        app.set_global::<u32>(1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            app.with_global_mut(
                || 0u32,
                |_v, app| {
                    let _ = app.global::<u32>();
                },
            );
        }));
        assert!(result.is_err());
    }

    #[test]
    fn set_global_panics_while_leased() {
        let mut app = App::new();
        app.set_global::<u32>(1);

        let result = catch_unwind(AssertUnwindSafe(|| {
            app.with_global_mut(
                || 0u32,
                |_v, app| {
                    app.set_global::<u32>(2);
                },
            );
        }));
        assert!(result.is_err());
    }

    #[test]
    fn global_changes_are_tracked_and_deduped() {
        let mut app = App::new();
        let _ = app.take_changed_globals();

        app.set_global::<u32>(1);
        let changed = app.take_changed_globals();
        assert_eq!(changed, vec![TypeId::of::<u32>()]);

        app.set_global::<u32>(2);
        app.set_global::<u32>(3);
        let changed = app.take_changed_globals();
        assert_eq!(changed, vec![TypeId::of::<u32>()]);

        app.with_global_mut(
            || 0u32,
            |v, _app| {
                *v = 4;
            },
        );
        let changed = app.take_changed_globals();
        assert_eq!(changed, vec![TypeId::of::<u32>()]);
    }

    #[test]
    fn global_revision_bumps_on_tracked_mutations_but_not_untracked() {
        let mut app = App::new();

        let ty = TypeId::of::<u32>();
        assert_eq!(app.global_revision(ty), None);

        app.set_global::<u32>(1);
        let r1 = app
            .global_revision(ty)
            .expect("global revision must exist after set");

        app.with_global_mut(
            || 0u32,
            |v, _app| {
                *v = v.saturating_add(1);
            },
        );
        let r2 = app
            .global_revision(ty)
            .expect("global revision must exist after mutation");
        assert!(r2 > r1);

        app.with_global_mut_untracked(
            || 0u32,
            |v, _app| {
                *v = v.saturating_add(1);
            },
        );
        let r3 = app
            .global_revision(ty)
            .expect("global revision must still exist");
        assert_eq!(r3, r2);
    }

    #[test]
    fn default_keymap_includes_undo_redo() {
        let service = default_keymap_service();
        let ctx = |platform: Platform| InputContext {
            platform,
            caps: Default::default(),
            ui_has_modal: false,
            window_arbitration: None,
            focus_is_text_input: false,
            text_boundary_mode: fret_runtime::TextBoundaryMode::UnicodeWord,
            edit_can_undo: true,
            edit_can_redo: true,
            router_can_back: false,
            router_can_forward: false,
            dispatch_phase: InputDispatchPhase::Bubble,
        };

        let ctrl_z = KeyChord::new(
            KeyCode::KeyZ,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let ctrl_y = KeyChord::new(
            KeyCode::KeyY,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        );
        let ctrl_shift_z = KeyChord::new(
            KeyCode::KeyZ,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        );
        let cmd_z = KeyChord::new(
            KeyCode::KeyZ,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        );
        let cmd_shift_z = KeyChord::new(
            KeyCode::KeyZ,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        );

        for platform in [Platform::Windows, Platform::Linux, Platform::Web] {
            assert_eq!(
                service.keymap.resolve(&ctx(platform), ctrl_z),
                Some(CommandId::from("edit.undo"))
            );
            assert_eq!(
                service.keymap.resolve(&ctx(platform), ctrl_y),
                Some(CommandId::from("edit.redo"))
            );
            assert_eq!(
                service.keymap.resolve(&ctx(platform), ctrl_shift_z),
                Some(CommandId::from("edit.redo"))
            );
        }

        assert_eq!(
            service.keymap.resolve(&ctx(Platform::Macos), cmd_z),
            Some(CommandId::from("edit.undo"))
        );
        assert_eq!(
            service.keymap.resolve(&ctx(Platform::Macos), cmd_shift_z),
            Some(CommandId::from("edit.redo"))
        );
    }

    #[test]
    fn app_new_installs_default_i18n_service() {
        let app = App::new();
        let service = app
            .global::<fret_i18n::I18nService>()
            .expect("i18n service should exist by default");
        assert_eq!(
            service.preferred_locales(),
            &[fret_i18n::LocaleId::default()]
        );
    }
}
