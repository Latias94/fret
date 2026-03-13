use super::super::*;

#[cfg(test)]
pub(in super::super) fn select_inserted_node_in_view_state(
    view_state: &mut NodeGraphViewState,
    node_id: GraphNodeId,
) {
    insert_execution_feedback::select_inserted_node_in_view_state(view_state, node_id)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn split_edge_candidate_rejection_toast(
        candidate: &InsertNodeCandidate,
        diags: &[Diagnostic],
    ) -> (DiagnosticSeverity, Arc<str>) {
        insert_execution_feedback::split_edge_candidate_rejection_toast::<M>(candidate, diags)
    }

    pub(in super::super) fn select_inserted_node<H: UiHost>(
        &mut self,
        host: &mut H,
        node_id: Option<GraphNodeId>,
    ) {
        insert_execution_feedback::select_inserted_node(self, host, node_id)
    }
}
