use super::*;

pub(super) fn handle_pan_inertia_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) -> bool {
    super::timer_motion_pan_inertia::handle_pan_inertia_tick(canvas, cx, snapshot, token)
}

pub(super) fn handle_viewport_animation_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    super::timer_motion_viewport::handle_viewport_animation_tick(canvas, cx, token)
}

pub(super) fn handle_auto_pan_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    token: fret_core::TimerToken,
) -> bool {
    super::timer_motion_auto_pan::handle_auto_pan_tick(canvas, cx, snapshot, token)
}

pub(super) fn handle_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    super::timer_motion_viewport::handle_viewport_move_debounce(canvas, cx, token)
}
