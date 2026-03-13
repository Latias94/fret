use super::*;

pub(super) fn edge_is_deletable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    edge: EdgeId,
) -> bool {
    if !interaction.edges_deletable {
        return false;
    }
    let Some(edge) = graph.edges.get(&edge) else {
        return false;
    };
    edge.deletable.unwrap_or(true)
}

pub(super) fn node_is_deletable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    node: GraphNodeId,
) -> bool {
    if !interaction.nodes_deletable {
        return false;
    }
    let Some(node) = graph.nodes.get(&node) else {
        return false;
    };
    node.deletable.unwrap_or(true)
}
