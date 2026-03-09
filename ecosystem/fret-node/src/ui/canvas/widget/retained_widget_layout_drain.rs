use super::*;

pub(super) fn drain_post_layout_queues<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    canvas.drain_edit_queue(cx.app, cx.window);
    let did_view_queue = canvas.drain_view_queue(cx.app, cx.window);
    let did_fit_on_mount =
        canvas.maybe_fit_view_on_mount(cx.app, cx.window, cx.bounds, did_view_queue);
    if did_view_queue || did_fit_on_mount {
        cx.request_redraw();
    }
    cx.available
}
