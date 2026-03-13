use super::*;

pub(super) fn route_timer_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) {
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
