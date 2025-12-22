use std::any::Any;

use fret_app::{CommandRegistry, DragKind, DragSession, Effect, ModelId, ModelStore};
use fret_core::{AppWindowId, FrameId, Point, TickId, TimerToken};

pub trait UiHost {
    fn set_global<T: Any>(&mut self, value: T);
    fn global<T: Any>(&self) -> Option<&T>;
    fn global_mut<T: Any>(&mut self) -> Option<&mut T>;

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R;

    fn models(&self) -> &ModelStore;
    fn models_mut(&mut self) -> &mut ModelStore;
    fn take_changed_models(&mut self) -> Vec<ModelId>;

    fn commands(&self) -> &CommandRegistry;

    fn request_redraw(&mut self, window: AppWindowId);
    fn push_effect(&mut self, effect: Effect);

    fn tick_id(&self) -> TickId;
    fn frame_id(&self) -> FrameId;
    fn next_timer_token(&mut self) -> TimerToken;

    fn drag(&self) -> Option<&DragSession>;
    fn drag_mut(&mut self) -> Option<&mut DragSession>;
    fn cancel_drag(&mut self);

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    );

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    );
}

impl UiHost for fret_app::App {
    fn set_global<T: Any>(&mut self, value: T) {
        fret_app::App::set_global(self, value);
    }

    fn global<T: Any>(&self) -> Option<&T> {
        fret_app::App::global(self)
    }

    fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        fret_app::App::global_mut(self)
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        fret_app::App::with_global_mut(self, init, f)
    }

    fn models(&self) -> &ModelStore {
        fret_app::App::models(self)
    }

    fn models_mut(&mut self) -> &mut ModelStore {
        fret_app::App::models_mut(self)
    }

    fn take_changed_models(&mut self) -> Vec<ModelId> {
        fret_app::App::take_changed_models(self)
    }

    fn commands(&self) -> &CommandRegistry {
        fret_app::App::commands(self)
    }

    fn request_redraw(&mut self, window: AppWindowId) {
        fret_app::App::request_redraw(self, window);
    }

    fn push_effect(&mut self, effect: Effect) {
        fret_app::App::push_effect(self, effect);
    }

    fn tick_id(&self) -> TickId {
        fret_app::App::tick_id(self)
    }

    fn frame_id(&self) -> FrameId {
        fret_app::App::frame_id(self)
    }

    fn next_timer_token(&mut self) -> TimerToken {
        fret_app::App::next_timer_token(self)
    }

    fn drag(&self) -> Option<&DragSession> {
        fret_app::App::drag(self)
    }

    fn drag_mut(&mut self) -> Option<&mut DragSession> {
        fret_app::App::drag_mut(self)
    }

    fn cancel_drag(&mut self) {
        fret_app::App::cancel_drag(self)
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        fret_app::App::begin_drag_with_kind(self, kind, source_window, start, payload)
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        fret_app::App::begin_cross_window_drag_with_kind(self, kind, source_window, start, payload)
    }
}
