use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{AppWindowId, FrameId, NodeId, TickId, TimerToken};

use crate::drag::DragKind;
use crate::drag::DragSession;
use fret_runtime::{
    BindingV1, CommandRegistry, Effect, KeySpecV1, Keymap, KeymapFileV1, KeymapService, ModelHost,
    ModelId, ModelStore,
};

pub struct App {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw_requests: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drag: Option<DragSession>,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
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
            models: ModelStore::default(),
            commands: CommandRegistry::default(),
            redraw_requests: HashSet::new(),
            effects: Vec::new(),
            drag: None,
            tick_id: TickId::default(),
            frame_id: FrameId::default(),
            next_timer_token: 1,
        };

        // Provide a minimal default keymap so basic UI focus traversal works out of the box.
        // Apps can fully override this by installing their own `KeymapService`.
        app.set_global(default_keymap_service());

        app
    }

    pub fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    pub fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref::<T>())
    }

    pub fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.globals
            .get_mut(&TypeId::of::<T>())
            .and_then(|value| value.downcast_mut::<T>())
    }

    #[track_caller]
    pub fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut App) -> R,
    ) -> R {
        #[derive(Debug)]
        struct GlobalLeaseMarker;

        struct Guard<T: Any> {
            type_id: TypeId,
            value: Option<T>,
            globals: *mut HashMap<TypeId, Box<dyn Any>>,
        }

        impl<T: Any> Drop for Guard<T> {
            fn drop(&mut self) {
                let Some(value) = self.value.take() else {
                    return;
                };
                // Safety: this guard is only constructed from `App::with_global_mut`, and it
                // outlives the closure execution; `globals` remains valid for the duration.
                unsafe {
                    (*self.globals).insert(self.type_id, Box::new(value));
                }
            }
        }

        let type_id = TypeId::of::<T>();
        let existing = self
            .globals
            .insert(type_id, Box::new(GlobalLeaseMarker) as Box<dyn Any>);

        let existing = match existing {
            None => None,
            Some(v) => {
                if v.is::<GlobalLeaseMarker>() {
                    panic!(
                        "global already leased: {} ({type_id:?}) at {}",
                        std::any::type_name::<T>(),
                        std::panic::Location::caller()
                    );
                }
                Some(*v.downcast::<T>().expect("global type id must match"))
            }
        };

        let mut guard = Guard::<T> {
            type_id,
            value: Some(existing.unwrap_or_else(init)),
            globals: &mut self.globals as *mut _,
        };

        // Safety: we keep `T` out of `self.globals` until `guard` drops and reinserts it,
        // so it is safe to pass both `&mut T` and `&mut App` to the callback.
        let result = {
            let value = guard.value.as_mut().expect("guard value exists");
            f(value, self)
        };

        drop(guard);
        result
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
