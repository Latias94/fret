use super::*;

pub(super) fn split_edge_candidate_rejection_toast<M: NodeGraphCanvasMiddleware>(
    candidate: &InsertNodeCandidate,
    diags: &[Diagnostic],
) -> (DiagnosticSeverity, Arc<str>) {
    NodeGraphCanvasWith::<M>::toast_from_diagnostics(diags).unwrap_or_else(|| {
        (
            DiagnosticSeverity::Error,
            Arc::<str>::from(format!("node insertion was rejected: {}", candidate.kind.0)),
        )
    })
}

pub(super) fn select_inserted_node_in_view_state(
    view_state: &mut NodeGraphViewState,
    node_id: GraphNodeId,
) {
    view_state.selected_edges.clear();
    view_state.selected_groups.clear();
    view_state.selected_nodes.clear();
    view_state.selected_nodes.push(node_id);
    view_state.draw_order.retain(|id| *id != node_id);
    view_state.draw_order.push(node_id);
}

pub(super) fn select_inserted_node<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    node_id: Option<GraphNodeId>,
) {
    if let Some(node_id) = node_id {
        canvas.update_view_state(host, |view_state| {
            select_inserted_node_in_view_state(view_state, node_id);
        });
    }
}
