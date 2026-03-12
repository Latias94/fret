use super::super::*;

pub(in super::super) fn graph_node_is_draggable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    node: GraphNodeId,
) -> bool {
    if !interaction.nodes_draggable {
        return false;
    }

    let Some(node) = graph.nodes.get(&node) else {
        return false;
    };
    node.draggable.unwrap_or(true)
}

pub(in super::super) fn graph_node_is_connectable(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    node: GraphNodeId,
) -> bool {
    let Some(node) = graph.nodes.get(&node) else {
        return false;
    };
    node.connectable.unwrap_or(interaction.nodes_connectable)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn node_is_draggable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        graph_node_is_draggable(graph, interaction, node)
    }
}

#[cfg(test)]
mod tests;
