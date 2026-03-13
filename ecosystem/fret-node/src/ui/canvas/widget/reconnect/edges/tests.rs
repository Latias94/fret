use super::*;
use crate::core::{
    Edge, EdgeKind, GraphId, NodeId as GraphNodeId, Port, PortCapacity, PortDirection, PortKey,
    PortKind,
};
use serde_json::Value;

fn sample_graph() -> (Graph, PortId, PortId, PortId, EdgeId, EdgeId) {
    let graph_id = GraphId::from_u128(1);
    let source_node = GraphNodeId::from_u128(10);
    let target_node = GraphNodeId::from_u128(11);
    let other_node = GraphNodeId::from_u128(12);
    let source_port = PortId::from_u128(20);
    let target_port = PortId::from_u128(21);
    let other_port = PortId::from_u128(22);
    let outgoing_edge = EdgeId::from_u128(30);
    let incoming_edge = EdgeId::from_u128(31);

    let mut graph = Graph::new(graph_id);
    graph.ports.insert(
        source_port,
        Port {
            node: source_node,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
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
        target_port,
        Port {
            node: target_node,
            key: PortKey::new("in"),
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
        other_port,
        Port {
            node: other_node,
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
    graph.edges.insert(
        outgoing_edge,
        Edge {
            kind: EdgeKind::Data,
            from: source_port,
            to: target_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph.edges.insert(
        incoming_edge,
        Edge {
            kind: EdgeKind::Data,
            from: source_port,
            to: other_port,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    (
        graph,
        source_port,
        target_port,
        other_port,
        outgoing_edge,
        incoming_edge,
    )
}

#[test]
fn yank_edges_from_out_port_marks_from_endpoint_and_fixed_peer() {
    let (graph, source_port, target_port, other_port, outgoing_edge, incoming_edge) =
        sample_graph();

    let edges = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::yank_edges_from_port(
        &graph,
        source_port,
    );

    assert_eq!(
        edges,
        vec![
            (outgoing_edge, EdgeEndpoint::From, target_port),
            (incoming_edge, EdgeEndpoint::From, other_port),
        ]
    );
}

#[test]
fn yank_edges_from_in_port_marks_to_endpoint_and_fixed_peer() {
    let (graph, source_port, target_port, _other_port, outgoing_edge, _incoming_edge) =
        sample_graph();

    let edges = NodeGraphCanvasWith::<NoopNodeGraphCanvasMiddleware>::yank_edges_from_port(
        &graph,
        target_port,
    );

    assert_eq!(edges, vec![(outgoing_edge, EdgeEndpoint::To, source_port)]);
}
