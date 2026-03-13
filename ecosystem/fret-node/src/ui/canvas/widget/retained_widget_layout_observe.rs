use super::*;

pub(super) fn observe_layout_models<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    cx: &mut LayoutCx<'_, H>,
) {
    cx.observe_model(&canvas.graph, Invalidation::Layout);
    cx.observe_model(&canvas.view_state, Invalidation::Layout);
    if let Some(queue) = canvas.edit_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
    if let Some(queue) = canvas.view_queue.as_ref() {
        cx.observe_model(queue, Invalidation::Layout);
    }
}
