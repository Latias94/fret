use crate::ui::canvas::widget::*;

pub(super) fn plan_split_edge_reroute_with_graph(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    edge_id: EdgeId,
    at: CanvasPoint,
) -> Result<Vec<GraphOp>, Vec<Diagnostic>> {
    let plan = presenter.plan_split_edge(graph, edge_id, &NodeKindKey::new(REROUTE_KIND), at);
    match plan.decision {
        ConnectDecision::Accept => Ok(plan.ops),
        ConnectDecision::Reject => Err(plan.diagnostics),
    }
}

pub(super) fn plan_canvas_split_edge_reroute<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    edge_id: EdgeId,
    invoked_at: Point,
) -> Option<Result<Vec<GraphOp>, Vec<Diagnostic>>> {
    let at = canvas.reroute_pos_for_invoked_at(invoked_at);
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            plan_split_edge_reroute_with_graph(presenter, graph, edge_id, at)
        })
        .ok()
}
