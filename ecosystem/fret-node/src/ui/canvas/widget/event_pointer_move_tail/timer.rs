use super::*;

pub(super) fn sync_pointer_move_auto_pan_timer<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) {
    let snapshot = canvas.sync_view_state(cx.app);
    canvas.sync_auto_pan_timer(cx.app, cx.window, &snapshot, cx.bounds);
}
