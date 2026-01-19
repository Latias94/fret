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
        selectable: None,
        draggable: None,
        connectable: None,
        deletable: None,
        parent: None,
        extent: None,
        expand_parent: None,
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
        connectable: None,
        connectable_start: None,
        connectable_end: None,
        ty: None,
        data: serde_json::Value::Null,
    }
}

#[test]
fn validate_allows_edges_regardless_of_port_direction() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out_a = PortId::new();
    let out_b = PortId::new();
    graph.ports.insert(
        out_a,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );
    graph.ports.insert(
        out_b,
        make_port(
            b,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
        ),
    );

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out_a,
            to: out_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let report = validate_graph(&graph);
    assert!(report.is_ok());
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

#[test]
fn validate_rejects_edge_kind_that_does_not_match_port_kind() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out_a = PortId::new();
    let in_b = PortId::new();
    graph.ports.insert(
        out_a,
        make_port(
            a,
            "out",
            PortDirection::Out,
            PortKind::Data,
            PortCapacity::Multi,
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
            kind: EdgeKind::Exec,
            from: out_a,
            to: in_b,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let report = validate_graph(&graph);
    assert!(!report.is_ok());
}
