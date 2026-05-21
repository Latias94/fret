use super::*;

pub(super) fn route_timer_tick<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) where
    M: NodeGraphCanvasMiddleware,
    Cx: timer_motion_cx::TimerMotionCx<H>,
{
    if timer_motion::handle_pan_inertia_tick(canvas, cx, snapshot, token) {
        return;
    }

    if timer_motion::handle_viewport_animation_tick(canvas, cx, token) {
        return;
    }

    if timer_motion::handle_auto_pan_tick(canvas, cx, snapshot, token) {
        return;
    }

    let _ = timer_motion::handle_viewport_move_debounce(canvas, cx, token);
}
