use std::collections::BTreeSet;

use super::super::*;

pub(in super::super) fn push_edge_remove_ops(
    ops: &mut Vec<GraphOp>,
    removed_edges: &BTreeSet<EdgeId>,
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    selected_edges: &[EdgeId],
) {
    let mut edges_sel: Vec<EdgeId> = selected_edges.to_vec();
    edges_sel.sort();
    for edge_id in edges_sel {
        if removed_edges.contains(&edge_id) {
            continue;
        }
        if !super::super::delete_predicates::edge_is_deletable(graph, interaction, edge_id) {
            continue;
        }
        let Some(edge) = graph.edges.get(&edge_id) else {
            continue;
        };
        ops.push(GraphOp::RemoveEdge {
            id: edge_id,
            edge: edge.clone(),
        });
    }
}
