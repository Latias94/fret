use crate::ui::canvas::widget::*;

pub(super) fn open_edge_insert_context_menu<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge_id: EdgeId,
    invoked_at: Point,
) {
    edge_insert::open_edge_insert_context_menu(canvas, cx, edge_id, invoked_at);
}
