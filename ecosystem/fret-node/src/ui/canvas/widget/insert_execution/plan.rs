use super::super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn plan_insert_candidate_ops_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Arc<str>> {
        insert_execution_plan::plan_insert_candidate_ops_with_graph::<M>(
            presenter, graph, candidate, at,
        )
    }

    pub(in super::super) fn plan_canvas_insert_candidate_ops<H: UiHost>(
        &mut self,
        host: &mut H,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Option<Result<Vec<GraphOp>, Arc<str>>> {
        insert_execution_plan::plan_canvas_insert_candidate_ops(self, host, candidate, at)
    }

    pub(in super::super) fn plan_split_edge_insert_candidate_with_graph(
        presenter: &mut dyn NodeGraphPresenter,
        graph: &Graph,
        edge_id: EdgeId,
        candidate: &InsertNodeCandidate,
        at: CanvasPoint,
    ) -> Result<Vec<GraphOp>, Vec<Diagnostic>> {
        insert_execution_plan::plan_split_edge_insert_candidate_with_graph(
            presenter, graph, edge_id, candidate, at,
        )
    }

    pub(in super::super) fn plan_canvas_split_edge_insert_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
        candidate: &InsertNodeCandidate,
        invoked_at: Point,
    ) -> Option<Result<Vec<GraphOp>, Vec<Diagnostic>>> {
        insert_execution_plan::plan_canvas_split_edge_insert_candidate(
            self, host, edge_id, candidate, invoked_at,
        )
    }

    pub(in super::super) fn can_split_edge_insert_candidate<H: UiHost>(
        &mut self,
        host: &mut H,
        edge_id: EdgeId,
        candidate: &InsertNodeCandidate,
        invoked_at: Point,
    ) -> Option<bool> {
        insert_execution_plan::can_split_edge_insert_candidate(
            self, host, edge_id, candidate, invoked_at,
        )
    }
}
