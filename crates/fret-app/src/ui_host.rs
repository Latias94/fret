use std::any::Any;

use fret_core::{AppWindowId, Point};
use fret_runtime::{
    CommandsHost, DragHost, DragKind, DragSession, Effect, EffectSink, GlobalsHost, ModelsHost,
    TimeHost,
};

use crate::App;

impl GlobalsHost for App {
    fn set_global<T: Any>(&mut self, value: T) {
        App::set_global(self, value);
    }

    fn global<T: Any>(&self) -> Option<&T> {
        App::global(self)
    }

    fn global_mut<T: Any>(&mut self) -> Option<&mut T> {
        App::global_mut(self)
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        App::with_global_mut(self, init, f)
    }
}

impl ModelsHost for App {
    fn take_changed_models(&mut self) -> Vec<fret_runtime::ModelId> {
        App::take_changed_models(self)
    }
}

impl CommandsHost for App {
    fn commands(&self) -> &fret_runtime::CommandRegistry {
        App::commands(self)
    }
}

impl EffectSink for App {
    fn request_redraw(&mut self, window: AppWindowId) {
        App::request_redraw(self, window);
    }

    fn push_effect(&mut self, effect: Effect) {
        App::push_effect(self, effect);
    }
}

impl TimeHost for App {
    fn tick_id(&self) -> fret_runtime::TickId {
        App::tick_id(self)
    }

    fn frame_id(&self) -> fret_core::FrameId {
        App::frame_id(self)
    }

    fn next_timer_token(&mut self) -> fret_core::TimerToken {
        App::next_timer_token(self)
    }

    fn next_clipboard_token(&mut self) -> fret_core::ClipboardToken {
        App::next_clipboard_token(self)
    }
}

impl DragHost for App {
    fn drag(&self) -> Option<&DragSession> {
        App::drag(self)
    }

    fn drag_mut(&mut self) -> Option<&mut DragSession> {
        App::drag_mut(self)
    }

    fn cancel_drag(&mut self) {
        App::cancel_drag(self)
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        App::begin_drag_with_kind(self, kind, source_window, start, payload)
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        kind: DragKind,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        App::begin_cross_window_drag_with_kind(self, kind, source_window, start, payload)
    }
}
