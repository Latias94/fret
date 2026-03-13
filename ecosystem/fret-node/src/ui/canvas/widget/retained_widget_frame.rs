use super::*;

pub(super) fn sync_semantics<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut SemanticsCx<'_, H>,
) {
    super::retained_widget_semantics::sync_semantics(canvas, cx);
}

pub(super) fn layout_widget<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) -> Size {
    super::retained_widget_layout::layout_widget(canvas, cx)
}

pub(super) fn prepaint_cull_window<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut PrepaintCx<'_, H>,
) {
    super::retained_widget_cull_window::prepaint_cull_window(canvas, cx);
}
