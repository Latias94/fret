use std::any::Any;

use fret_core::{AppWindowId, Point, PointerId};
use fret_runtime::{
    CommandsHost, DragHost, DragKindId, DragSession, Effect, EffectSink, GlobalsHost, ModelsHost,
    TimeHost,
};

use crate::App;

impl GlobalsHost for App {
    #[track_caller]
    fn set_global<T: Any>(&mut self, value: T) {
        let at = std::panic::Location::caller();
        App::set_global_at(self, value, at);
    }

    fn global<T: Any>(&self) -> Option<&T> {
        App::global(self)
    }

    #[track_caller]
    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let at = std::panic::Location::caller();
        App::with_global_mut_impl(self, init, f, at, true)
    }

    #[track_caller]
    fn with_global_mut_untracked<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        let at = std::panic::Location::caller();
        App::with_global_mut_impl(self, init, f, at, false)
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

impl fret_runtime::InboxDrainHost for App {
    fn request_redraw(&mut self, window: AppWindowId) {
        App::request_redraw(self, window);
    }

    fn push_effect(&mut self, effect: Effect) {
        App::push_effect(self, effect);
    }

    fn models_mut(&mut self) -> &mut fret_runtime::ModelStore {
        App::models_mut(self)
    }
}

impl TimeHost for App {
    fn tick_id(&self) -> fret_runtime::TickId {
        App::tick_id(self)
    }

    fn frame_id(&self) -> fret_runtime::FrameId {
        App::frame_id(self)
    }

    fn next_timer_token(&mut self) -> fret_runtime::TimerToken {
        App::next_timer_token(self)
    }

    fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
        App::next_clipboard_token(self)
    }

    fn next_image_upload_token(&mut self) -> fret_runtime::ImageUploadToken {
        App::next_image_upload_token(self)
    }
}

impl DragHost for App {
    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession> {
        App::drag(self, pointer_id)
    }

    fn any_drag_session(&self, mut predicate: impl FnMut(&DragSession) -> bool) -> bool {
        App::drags(self).any(|d| predicate(d))
    }

    fn find_drag_pointer_id(
        &self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<PointerId> {
        App::drags(self)
            .find(|d| predicate(d))
            .map(|d| d.pointer_id)
    }

    fn cancel_drag_sessions(
        &mut self,
        mut predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<PointerId> {
        let to_cancel: Vec<PointerId> = self
            .drags()
            .filter(|d| predicate(d))
            .map(|d| d.pointer_id)
            .collect();
        for pointer_id in &to_cancel {
            self.cancel_drag(*pointer_id);
        }
        to_cancel
    }

    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession> {
        App::drag_mut(self, pointer_id)
    }

    fn cancel_drag(&mut self, pointer_id: PointerId) {
        App::cancel_drag(self, pointer_id)
    }

    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        App::begin_drag_with_kind(self, pointer_id, kind, source_window, start, payload)
    }

    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    ) {
        App::begin_cross_window_drag_with_kind(
            self,
            pointer_id,
            kind,
            source_window,
            start,
            payload,
        )
    }
}
