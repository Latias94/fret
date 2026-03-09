use super::*;

pub(super) fn request_paste_feedback<H: UiHost>(cx: &mut EventCx<'_, H>) {
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
}

pub(super) fn show_clipboard_unavailable_toast<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) {
    canvas.show_toast(
        cx.app,
        cx.window,
        DiagnosticSeverity::Info,
        "clipboard text unavailable",
    );
    request_paste_feedback(cx);
}
