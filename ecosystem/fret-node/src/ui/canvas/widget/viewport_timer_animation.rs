mod animation;
mod debounce;

use super::*;
use crate::ui::canvas::state::{ViewportAnimationEase, ViewportAnimationInterpolate};

pub(super) fn stop_viewport_animation_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
) {
    animation::stop_viewport_animation_timer(canvas, host);
}

pub(super) fn start_viewport_animation_to<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    from_pan: CanvasPoint,
    from_zoom: f32,
    to_pan: CanvasPoint,
    to_zoom: f32,
    duration: std::time::Duration,
    interpolate: ViewportAnimationInterpolate,
    ease: Option<ViewportAnimationEase>,
) -> bool {
    animation::start_viewport_animation_to(
        canvas,
        host,
        window,
        from_pan,
        from_zoom,
        to_pan,
        to_zoom,
        duration,
        interpolate,
        ease,
    )
}

pub(super) fn bump_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    window: Option<AppWindowId>,
    snapshot: &ViewSnapshot,
    kind: ViewportMoveKind,
) {
    debounce::bump_viewport_move_debounce(canvas, host, window, snapshot, kind);
}
