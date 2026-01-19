use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(super) fn node_is_draggable(
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

    pub(super) fn port_is_connectable_base(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port) = graph.ports.get(&port) else {
            return false;
        };
        let node_connectable = Self::node_is_connectable(graph, interaction, port.node);
        port.connectable.unwrap_or(node_connectable)
    }

    pub(super) fn port_is_connectable_start(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_start.unwrap_or(true)
    }

    pub(super) fn port_is_connectable_end(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        let Some(port_value) = graph.ports.get(&port) else {
            return false;
        };
        if !Self::port_is_connectable_base(graph, interaction, port) {
            return false;
        }
        port_value.connectable_end.unwrap_or(true)
    }

    pub(super) fn node_is_connectable(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        node: GraphNodeId,
    ) -> bool {
        let Some(node) = graph.nodes.get(&node) else {
            return false;
        };
        node.connectable.unwrap_or(interaction.nodes_connectable)
    }

    pub(super) fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        if candidate == from || bundle.contains(&candidate) {
            return false;
        }
        let Some(from_port) = graph.ports.get(&from) else {
            return false;
        };
        let Some(candidate_port) = graph.ports.get(&candidate) else {
            return false;
        };
        candidate_port.dir == from_port.dir
    }
}
