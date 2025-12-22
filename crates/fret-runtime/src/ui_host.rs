use std::any::Any;

use fret_core::{AppWindowId, FrameId, Point, TickId, TimerToken};

use crate::{CommandRegistry, DragKind, DragSession, Effect, ModelId, ModelStore};

/// Host services required by the retained UI runtime (`fret-ui`).
///
/// This trait is intentionally minimal and portable: it lives in `fret-runtime` so that third-party
/// engines/editors can embed `fret-ui` without adopting `fret-app`.
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
