use super::super::*;

fn graph_edge_is_selectable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    edge: EdgeId,
) -> bool {
    if !interaction.elements_selectable || !interaction.edges_selectable {
        return false;
    }

    let Some(edge) = graph.edges.get(&edge) else {
        return false;
    };
    edge.selectable.unwrap_or(true)
}

fn graph_node_is_selectable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    node: GraphNodeId,
) -> bool {
    if !interaction.elements_selectable {
        return false;
    }

    let Some(node) = graph.nodes.get(&node) else {
        return false;
    };
    node.selectable.unwrap_or(true)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn edge_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        edge: EdgeId,
    ) -> bool {
        graph_edge_is_selectable(graph, interaction, edge)
    }

    pub(in super::super) fn node_is_selectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        graph_node_is_selectable(graph, interaction, node)
    }
}

#[cfg(test)]
mod tests;
