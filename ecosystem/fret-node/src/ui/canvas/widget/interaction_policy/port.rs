use super::super::*;

fn graph_port_is_connectable_base(
    graph: &Graph,
    interaction: &NodeGraphInteractionState,
    port: PortId,
) -> bool {
    let Some(port) = graph.ports.get(&port) else {
        return false;
    };
    let node_connectable = super::node::graph_node_is_connectable(graph, interaction, port.node);
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

fn should_add_bundle_port_with_graph(
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

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn port_is_connectable_base(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_base(graph, interaction, port)
    }

    pub(in super::super) fn port_is_connectable_start(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_start(graph, interaction, port)
    }

    pub(in super::super) fn port_is_connectable_end(
        graph: &Graph,
        interaction: &NodeGraphInteractionState,
        port: PortId,
    ) -> bool {
        graph_port_is_connectable_end(graph, interaction, port)
    }

    pub(in super::super) fn should_add_bundle_port(
        graph: &Graph,
        from: PortId,
        bundle: &[PortId],
        candidate: PortId,
    ) -> bool {
        should_add_bundle_port_with_graph(graph, from, bundle, candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        CanvasPoint, GraphId, Node, NodeKindKey, Port, PortCapacity, PortDirection, PortKey,
        PortKind,
    };
    use serde_json::Value;

    fn sample_graph() -> (Graph, PortId, PortId, PortId, PortId) {
        let graph_id = GraphId::from_u128(1);
        let node_a = GraphNodeId::from_u128(10);
        let node_b = GraphNodeId::from_u128(11);
        let from = PortId::from_u128(20);
        let candidate_same_dir = PortId::from_u128(21);
        let candidate_other_dir = PortId::from_u128(22);
        let node_default_port = PortId::from_u128(23);

        let mut graph = Graph::new(graph_id);
        graph.nodes.insert(
            node_a,
            Node {
                kind: NodeKindKey::new("test.a"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: Some(false),
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: vec![from, candidate_same_dir],
                data: Value::Null,
            },
        );
        graph.nodes.insert(
            node_b,
            Node {
                kind: NodeKindKey::new("test.b"),
                kind_version: 1,
                pos: CanvasPoint { x: 100.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
                collapsed: false,
                ports: vec![candidate_other_dir, node_default_port],
                data: Value::Null,
            },
        );
        graph.ports.insert(
            from,
            Port {
                node: node_a,
                key: PortKey::new("from"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: Some(true),
                connectable_start: Some(false),
                connectable_end: Some(true),
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            candidate_same_dir,
            Port {
                node: node_a,
                key: PortKey::new("same"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            candidate_other_dir,
            Port {
                node: node_b,
                key: PortKey::new("other"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: Value::Null,
            },
        );
        graph.ports.insert(
            node_default_port,
            Port {
                node: node_b,
                key: PortKey::new("default"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                connectable: None,
                connectable_start: None,
                connectable_end: Some(false),
                ty: None,
                data: Value::Null,
            },
        );

        (
            graph,
            from,
            candidate_same_dir,
            candidate_other_dir,
            node_default_port,
        )
    }

    #[test]
    fn port_connectable_helpers_respect_node_and_port_overrides() {
        let (graph, from, _candidate_same_dir, _candidate_other_dir, node_default_port) =
            sample_graph();
        let interaction = NodeGraphInteractionState::default();

        assert!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_base(
                &graph,
                &interaction,
                from,
            )
        );
        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_start(
                &graph,
                &interaction,
                from,
            )
        );
        assert!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_end(
                &graph,
                &interaction,
                from,
            )
        );

        assert!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_base(
                &graph,
                &interaction,
                node_default_port,
            )
        );
        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::port_is_connectable_end(
                &graph,
                &interaction,
                node_default_port,
            )
        );
    }

    #[test]
    fn should_add_bundle_port_requires_unique_same_direction_candidate() {
        let (graph, from, candidate_same_dir, candidate_other_dir, _node_default_port) =
            sample_graph();

        assert!(
            NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
                &graph,
                from,
                &[],
                candidate_same_dir,
            )
        );
        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
                &graph,
                from,
                &[],
                from,
            )
        );
        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
                &graph,
                from,
                &[candidate_same_dir],
                candidate_same_dir,
            )
        );
        assert!(
            !NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::should_add_bundle_port(
                &graph,
                from,
                &[],
                candidate_other_dir,
            )
        );
    }
}
