use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port, PortCapacity,
    PortDirection, PortId, PortKey, PortKind, validate_graph,
};
use crate::core::{CanvasSize, GraphValidationError, GroupId};

fn make_node(kind: &str) -> Node {
    Node {
        kind: NodeKindKey::new(kind),
        kind_version: 0,
        pos: CanvasPoint { x: 0.0, y: 0.0 },
        parent: None,
        size: None,
        collapsed: false,
        ports: Vec::new(),
        data: serde_json::Value::Null,
    }
}

fn make_port(
    node: NodeId,
    key: &str,
    dir: PortDirection,
    kind: PortKind,
    capacity: PortCapacity,
) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind,
        capacity,
        ty: None,
        data: serde_json::Value::Null,
    }
}

#[test]
fn validate_rejects_edge_with_wrong_direction() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let in_a = PortId::new();
    let in_b = PortId::new();
    graph.ports.insert(
        in_a,
        make_port(
            a,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );
    graph.ports.insert(
        in_b,
        make_port(
            b,
            "in",
            PortDirection::In,
            PortKind::Data,
            PortCapacity::Single,
        ),
    );

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: in_a,
            to: in_b,
            selectable: None,
        },
    );

    let report = validate_graph(&graph);
    assert!(!report.is_ok());
}

#[test]
fn validate_rejects_node_with_missing_parent_group() {
    let mut graph = Graph::default();
    let n = NodeId::new();
    let mut node = make_node("core.a");
    node.parent = Some(GroupId::new());
    graph.nodes.insert(n, node);

    let report = validate_graph(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::NodeParentMissingGroup { node, .. } if *node == n
    )));
}

#[test]
fn validate_rejects_node_with_invalid_size() {
    let mut graph = Graph::default();
    let n = NodeId::new();
    let mut node = make_node("core.a");
    node.size = Some(CanvasSize {
        width: -1.0,
        height: 10.0,
    });
    graph.nodes.insert(n, node);

    let report = validate_graph(&graph);
    assert!(report.errors.iter().any(|e| matches!(
        e,
        GraphValidationError::NodeInvalidSize { node, .. } if *node == n
    )));
}
