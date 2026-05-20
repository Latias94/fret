use super::*;

pub(super) fn layout_children<H: UiHost, M: NodeGraphCanvasMiddleware>(
    _canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    for &child in cx.children {
        cx.layout_in(child, cx.bounds);
    }
}
