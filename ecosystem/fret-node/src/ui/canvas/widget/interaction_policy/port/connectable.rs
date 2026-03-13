use super::super::super::*;

fn graph_port_is_connectable_base(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    port: PortId,
) -> bool {
    let Some(port) = graph.ports.get(&port) else {
        return false;
    };
    let node_connectable =
        super::super::node::graph_node_is_connectable(graph, interaction, port.node);
    port.connectable.unwrap_or(node_connectable)
}

fn graph_port_is_connectable_start(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    port: PortId,
) -> bool {
    let Some(port_value) = graph.ports.get(&port) else {
        return false;
    };
    if !graph_port_is_connectable_base(graph, interaction, port) {
        return false;
    }
    port_value.connectable_start.unwrap_or(true)
}

fn graph_port_is_connectable_end(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    port: PortId,
) -> bool {
    let Some(port_value) = graph.ports.get(&port) else {
        return false;
    };
    if !graph_port_is_connectable_base(graph, interaction, port) {
        return false;
    }
    port_value.connectable_end.unwrap_or(true)
}

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super::super) fn port_is_connectable_base(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_base(graph, interaction, port)
    }

    pub(in super::super::super) fn port_is_connectable_start(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_start(graph, interaction, port)
    }

    pub(in super::super::super) fn port_is_connectable_end(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_end(graph, interaction, port)
    }
}
