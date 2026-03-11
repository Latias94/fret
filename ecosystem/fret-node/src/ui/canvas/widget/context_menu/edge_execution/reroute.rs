use crate::ui::canvas::widget::*;

pub(super) fn insert_edge_reroute<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    edge_id: EdgeId,
    invoked_at: Point,
) {
    let outcome = canvas.plan_canvas_split_edge_reroute(cx.app, edge_id, invoked_at);
    canvas.execute_split_edge_reroute_outcome(cx.app, cx.window, None, outcome);
}
