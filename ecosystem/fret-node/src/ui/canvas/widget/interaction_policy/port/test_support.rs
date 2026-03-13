use super::super::super::*;
use crate::core::{
    CanvasPoint, GraphId, Node, NodeKindKey, Port, PortCapacity, PortDirection, PortKey, PortKind,
};
use serde_json::Value;

pub(super) fn sample_graph() -> (Graph, PortId, PortId, PortId, PortId) {
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
