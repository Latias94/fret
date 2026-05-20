use crate::ui::canvas::widget::*;

pub(super) fn insert_edge_reroute<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl super::EdgeContextActionCx<H>,
    edge_id: EdgeId,
    invoked_at: Point,
) {
    let outcome = canvas.plan_canvas_split_edge_reroute(cx.host(), edge_id, invoked_at);
    let window = cx.window();
    canvas.execute_split_edge_reroute_outcome(cx.host(), window, None, outcome);
}
