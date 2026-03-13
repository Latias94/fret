use super::*;

pub(super) fn plan_insert_candidate_ops_with_graph<M: NodeGraphCanvasMiddleware>(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    candidate: &InsertNodeCandidate,
    at: CanvasPoint,
) -> Result<Vec<GraphOp>, Arc<str>> {
    if super::insert_execution_point::is_reroute_insert_candidate(candidate) {
        Ok(NodeGraphCanvasWith::<M>::build_reroute_create_ops(at))
    } else {
        presenter.plan_create_node(graph, candidate, at)
    }
}

pub(super) fn plan_canvas_insert_candidate_ops<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    candidate: &InsertNodeCandidate,
    at: CanvasPoint,
) -> Option<Result<Vec<GraphOp>, Arc<str>>> {
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            NodeGraphCanvasWith::<M>::plan_insert_candidate_ops_with_graph(
                presenter, graph, candidate, at,
            )
        })
        .ok()
}

pub(super) fn plan_split_edge_insert_candidate_with_graph(
    presenter: &mut dyn NodeGraphPresenter,
    graph: &Graph,
    edge_id: EdgeId,
    candidate: &InsertNodeCandidate,
    at: CanvasPoint,
) -> Result<Vec<GraphOp>, Vec<Diagnostic>> {
    let plan = presenter.plan_split_edge_candidate(graph, edge_id, candidate, at);
    match plan.decision {
        ConnectDecision::Accept => Ok(plan.ops),
        ConnectDecision::Reject => Err(plan.diagnostics),
    }
}

pub(super) fn plan_canvas_split_edge_insert_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    edge_id: EdgeId,
    candidate: &InsertNodeCandidate,
    invoked_at: Point,
) -> Option<Result<Vec<GraphOp>, Vec<Diagnostic>>> {
    let at =
        super::insert_execution_point::insert_candidate_canvas_point(canvas, candidate, invoked_at);
    let presenter = &mut *canvas.presenter;
    canvas
        .graph
        .read_ref(host, |graph| {
            NodeGraphCanvasWith::<M>::plan_split_edge_insert_candidate_with_graph(
                presenter, graph, edge_id, candidate, at,
            )
        })
        .ok()
}

pub(super) fn can_split_edge_insert_candidate<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    edge_id: EdgeId,
    candidate: &InsertNodeCandidate,
    invoked_at: Point,
) -> Option<bool> {
    plan_canvas_split_edge_insert_candidate(canvas, host, edge_id, candidate, invoked_at)
        .map(|result| result.is_ok())
}
