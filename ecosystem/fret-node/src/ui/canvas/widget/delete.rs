use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn delete_selection_ops(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        selected_nodes: &[GraphNodeId],
        selected_edges: &[EdgeId],
        selected_groups: &[crate::core::GroupId],
    ) -> Vec<GraphOp> {
        super::delete_ops_builder::delete_selection_ops(
            graph,
            interaction,
            selected_nodes,
            selected_edges,
            selected_groups,
        )
    }

    pub(super) fn removed_ids_from_ops(
        ops: &[GraphOp],
    ) -> (
        HashSet<GraphNodeId>,
        HashSet<EdgeId>,
        HashSet<crate::core::GroupId>,
    ) {
        super::delete_removed_ids::removed_ids_from_ops(ops)
    }
}
