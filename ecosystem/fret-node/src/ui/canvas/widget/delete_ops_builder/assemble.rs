use std::collections::BTreeSet;

use super::super::*;

pub(in super::super) fn delete_selection_ops(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_nodes: &[GraphNodeId],
    selected_edges: &[EdgeId],
    selected_groups: &[crate::core::GroupId],
) -> Vec<GraphOp> {
    let mut ops: Vec<GraphOp> = Vec::new();
    let mut removed_edges: BTreeSet<EdgeId> = BTreeSet::new();

    super::group::push_group_remove_ops(&mut ops, graph, selected_groups);
    super::node::push_node_remove_ops(
        &mut ops,
        &mut removed_edges,
        graph,
        interaction,
        selected_nodes,
    );
    super::edge::push_edge_remove_ops(&mut ops, &removed_edges, graph, interaction, selected_edges);
    ops
}
