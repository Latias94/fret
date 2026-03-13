use std::collections::BTreeSet;

use super::super::super::*;

pub(super) fn collect_box_select_edges_from_graph_with_mode(
    graph: &Graph,
    mode: crate::io::NodeGraphBoxSelectEdges,
    nodes: &BTreeSet<GraphNodeId>,
) -> Vec<EdgeId> {
    graph
        .edges
        .iter()
        .filter_map(|(edge_id, edge)| {
            if !edge.selectable.unwrap_or(true) {
                return None;
            }
            let from_node = graph.ports.get(&edge.from).map(|port| port.node)?;
            let to_node = graph.ports.get(&edge.to).map(|port| port.node)?;
            super::mode::edge_matches_box_select_mode(mode, nodes, from_node, to_node)
                .then_some(*edge_id)
        })
        .collect()
}

#[cfg(test)]
pub(super) fn collect_box_select_edges_from_graph(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    nodes: &BTreeSet<GraphNodeId>,
) -> Vec<EdgeId> {
    let Some(mode) = super::mode::box_select_edge_mode(interaction) else {
        return Vec::new();
    };
    collect_box_select_edges_from_graph_with_mode(graph, mode, nodes)
}
