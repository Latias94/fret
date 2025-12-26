use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
};

use fret_core::{AppWindowId, FrameId, Point, TickId, TimerToken};
use fret_runtime::{CommandRegistry, DragKind, DragSession, Effect, ModelId, ModelStore, UiHost};

#[derive(Default)]
pub(crate) struct TestHost {
    globals: HashMap<TypeId, Box<dyn Any>>,
    models: ModelStore,
    commands: CommandRegistry,
    redraw: HashSet<AppWindowId>,
    effects: Vec<Effect>,
    drag: Option<DragSession>,
    tick_id: TickId,
    frame_id: FrameId,
    next_timer_token: u64,
}

impl UiHost for TestHost {
    fn set_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    fn global<T: Any>(&self) -> Option<&T> {
        self.globals
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
    }

    fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.globals
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        if self.global::<T>().is_none() {
            self.set_global(init());
        }

        let type_id = TypeId::of::<T>();
        let boxed_any = self.globals.remove(&type_id).expect("global exists");
        let mut boxed_t: Box<T> = boxed_any.downcast().ok().expect("type matches");

        let result = f(&mut boxed_t, self);
        self.globals.insert(type_id, boxed_t);
        result
    }

    fn models(&self) -> &ModelStore {
        &self.models
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        &mut self.models
    }

    fn take_changed_models(&mut self) -> Vec<ModelId> {
        self.models.take_changed_models()
    }

    fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        self.redraw.insert(window);
    }

    fn push_effect(&mut self, effect: Effect) {
        match effect {
            Effect::Redraw(window) => self.request_redraw(window),
            other => self.effects.push(other),
        }
    }

    fn tick_id(&self) -> TickId {
        self.tick_id
    }

    fn frame_id(&self) -> FrameId {
        self.frame_id
    }

    fn next_timer_token(&mut self) -> TimerToken {
        let token = TimerToken(self.next_timer_token);
        self.next_timer_token = self.next_timer_token.saturating_add(1);
        token
    }

    fn drag(&self) -> Option<&DragSession> {
        self.drag.as_ref()
    }

    fn drag_mut(&mut self) -> Option<&mut DragSession> {
        self.drag.as_mut()
    }

    fn cancel_drag(&mut self) {
        self.drag = None;
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new(source_window, kind, start, payload));
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        self.drag = Some(DragSession::new_cross_window(
            source_window,
            kind,
            start,
            payload,
        ));
    }
}

impl TestHost {
    pub(crate) fn effects(&self) -> &[Effect] {
        &self.effects
    }
}
