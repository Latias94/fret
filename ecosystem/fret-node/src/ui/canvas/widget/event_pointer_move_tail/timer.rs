use super::super::auto_pan_timer_cx::AutoPanTimerCx;
use super::*;

pub(super) fn sync_pointer_move_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl AutoPanTimerCx<H>,
) {
    let window = cx.window();
    let bounds = cx.bounds();
    let snapshot = canvas.sync_view_state(cx.host());
    canvas.sync_auto_pan_timer(cx.host(), window, &snapshot, bounds);
}
