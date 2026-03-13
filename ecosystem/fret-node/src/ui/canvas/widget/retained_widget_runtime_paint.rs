use super::*;

pub(super) fn paint_retained_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PaintCx<'_, H>,
) {
    super::retained_widget_runtime_shared::sync_runtime_theme(
        canvas,
        cx.theme().snapshot(),
        Some(cx.services),
    );
    canvas.paint_root(cx);
}
