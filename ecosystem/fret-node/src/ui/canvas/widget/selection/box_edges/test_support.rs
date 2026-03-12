use super::super::super::*;
use crate::core::{
    CanvasPoint, Edge, EdgeKind, GraphId, Node, NodeKindKey, Port, PortCapacity, PortDirection,
    PortKey, PortKind,
};
use serde_json::Value;

pub(super) fn sample_graph() -> (
    Graph,
    GraphNodeId,
    GraphNodeId,
    GraphNodeId,
    EdgeId,
    EdgeId,
    EdgeId,
) {
    let graph_id = GraphId::from_u128(1);
    let node_a = GraphNodeId::from_u128(10);
    let node_b = GraphNodeId::from_u128(11);
    let node_c = GraphNodeId::from_u128(12);
    let a_out = PortId::from_u128(20);
    let b_in = PortId::from_u128(21);
    let b_out = PortId::from_u128(22);
    let c_in = PortId::from_u128(23);
    let c_in_secondary = PortId::from_u128(24);
    let edge_ab = EdgeId::from_u128(30);
    let edge_bc = EdgeId::from_u128(31);
    let edge_ac_hidden = EdgeId::from_u128(32);

    let mut graph = Graph::new(graph_id);
    graph.nodes.insert(
        node_a,
        Node {
            kind: NodeKindKey::new("test.a"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
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
            ports: vec![a_out],
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
            ports: vec![b_in, b_out],
            data: Value::Null,
        },
    );
    graph.nodes.insert(
        node_c,
        Node {
            kind: NodeKindKey::new("test.c"),
            kind_version: 1,
            pos: CanvasPoint { x: 200.0, y: 0.0 },
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
            ports: vec![c_in, c_in_secondary],
            data: Value::Null,
        },
    );
    graph.ports.insert(
        a_out,
        Port {
            node: node_a,
            key: PortKey::new("a.out"),
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
        b_in,
        Port {
            node: node_b,
            key: PortKey::new("b.in"),
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
        b_out,
        Port {
            node: node_b,
            key: PortKey::new("b.out"),
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
        c_in,
        Port {
            node: node_c,
            key: PortKey::new("c.in"),
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
        c_in_secondary,
        Port {
            node: node_c,
            key: PortKey::new("c.in.secondary"),
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
    graph.edges.insert(
        edge_ab,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        edge_bc,
        Edge {
            kind: EdgeKind::Data,
            from: b_out,
            to: c_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        edge_ac_hidden,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: c_in_secondary,
            selectable: Some(false),
            deletable: None,
            reconnectable: None,
        },
    );

    (
        graph,
        node_a,
        node_b,
        node_c,
        edge_ab,
        edge_bc,
        edge_ac_hidden,
    )
}
