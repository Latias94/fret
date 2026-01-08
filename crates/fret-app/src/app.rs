use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
};

use fret_core::{AppWindowId, NodeId};
use fret_runtime::{ClipboardToken, FrameId, ImageUploadToken, TickId, TimerToken};

use crate::drag::DragKind;
use crate::drag::DragSession;
use fret_runtime::{
    BindingV1, CommandRegistry, Effect, KeySpecV1, Keymap, KeymapFileV1, KeymapService, ModelHost,
    ModelId, ModelStore,
};

#[derive(Debug)]
struct GlobalLeaseMarker {
    type_name: &'static str,
    leased_at: &'static std::panic::Location<'static>,
}

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    changed_globals: Vec<TypeId>,
    changed_globals_dedup: HashSet<TypeId>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw_requests: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drag: Option<DragSession>,
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
            changed_globals: Vec::new(),
            changed_globals_dedup: HashSet::new(),
            models: ModelStore::default(),
            commands: CommandRegistry::default(),
            redraw_requests: HashSet::new(),
            effects: Vec::new(),
            drag: None,
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

        app
    }

    fn mark_global_changed(&mut self, id: TypeId) {
        if self.changed_globals_dedup.insert(id) {
            self.changed_globals.push(id);
        }
    }

    pub fn take_changed_globals(&mut self) -> Vec<TypeId> {
        self.changed_globals_dedup.clear();
        std::mem::take(&mut self.changed_globals)
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
        let type_id = TypeId::of::<T>();
        if let Some(existing) = self.globals.get(&type_id) {
            Self::assert_global_not_leased::<T>(existing.as_ref());
        }
        self.globals.insert(type_id, Box::new(value));
        self.mark_global_changed(type_id);
    }

    #[track_caller]
    pub fn global<T: Any>(&self) -> Option<&T> {
        let value = self.globals.get(&TypeId::of::<T>())?;
        Self::assert_global_not_leased::<T>(value.as_ref());
        value.downcast_ref::<T>()
    }

    #[track_caller]
    pub fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        let value = self.globals.get_mut(&TypeId::of::<T>())?;
        Self::assert_global_not_leased::<T>(value.as_ref());
        value.downcast_mut::<T>()
    }

    #[track_caller]
    pub fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
    ) -> R {
        let type_id = TypeId::of::<T>();
        let marker = GlobalLeaseMarker {
            type_name: std::any::type_name::<T>(),
            leased_at: std::panic::Location::caller(),
        };
        let existing = self.globals.insert(type_id, Box::new(marker));

        let mut value = match existing {
            None => init(),
            Some(v) => {
                if let Some(marker) = v.downcast_ref::<GlobalLeaseMarker>() {
                    panic!(
                        "global already leased: {} (type_id={type_id:?}); leased at {}; accessed at {}",
                        marker.type_name,
                        marker.leased_at,
                        std::panic::Location::caller()
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

        self.mark_global_changed(type_id);

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

    pub fn begin_drag<T: Any>(
        &mut self,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        self.begin_drag_with_kind(DragKind::Custom, source_window, start, payload);
    }

    pub fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new(source_window, kind, start, payload));
    }

    pub fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: fret_core::Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new_cross_window(
            source_window,
            kind,
            start,
            payload,
        ));
    }

    pub fn drag(&self) -> Option<&DragSession> {
        self.drag.as_ref()
    }

    pub fn drag_mut(&mut self) -> Option<&mut DragSession> {
        self.drag.as_mut()
    }

    pub fn end_drag(&mut self) -> Option<DragSession> {
        self.drag.take()
    }

    pub fn cancel_drag(&mut self) {
        self.drag = None;
    }

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
        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            effect => self.effects.push(effect),
        }
    }

    pub fn flush_effects(&mut self) -> Vec<Effect> {
        let mut effects = std::mem::take(&mut self.effects);
        for window in self.redraw_requests.drain() {
            effects.push(Effect::Redraw(window));
        }
        effects
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
    use super::App;
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
}
