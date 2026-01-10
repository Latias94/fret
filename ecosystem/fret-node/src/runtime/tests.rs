use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port};
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};

fn make_graph() -> (
    Graph,
    NodeId,
    NodeId,
    crate::core::PortId,
    crate::core::PortId,
    EdgeId,
) {
    let mut g = Graph::new(crate::core::GraphId::from_u128(1));

    let a = NodeId::new();
    let b = NodeId::new();

    let out_port = crate::core::PortId::new();
    let in_port = crate::core::PortId::new();

    let node_a = Node {
        kind: NodeKindKey::new("demo.a"),
        kind_version: 1,
        pos: CanvasPoint { x: 0.0, y: 0.0 },
        parent: None,
        size: None,
        collapsed: false,
        ports: vec![out_port],
        data: serde_json::Value::Null,
    };
    let node_b = Node {
        kind: NodeKindKey::new("demo.b"),
        kind_version: 1,
        pos: CanvasPoint { x: 100.0, y: 0.0 },
        parent: None,
        size: None,
        collapsed: false,
        ports: vec![in_port],
        data: serde_json::Value::Null,
    };

    g.nodes.insert(a, node_a);
    g.nodes.insert(b, node_b);
    g.ports.insert(
        out_port,
        Port {
            node: a,
            key: crate::core::PortKey::new("out"),
            dir: crate::core::PortDirection::Out,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Multi,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    g.ports.insert(
        in_port,
        Port {
            node: b,
            key: crate::core::PortKey::new("in"),
            dir: crate::core::PortDirection::In,
            kind: crate::core::PortKind::Data,
            capacity: crate::core::PortCapacity::Single,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let eid = EdgeId::new();
    g.edges.insert(
        eid,
        Edge {
            kind: EdgeKind::Data,
            from: out_port,
            to: in_port,
        },
    );

    (g, a, b, out_port, in_port, eid)
}

#[test]
fn changes_from_transaction_maps_ops() {
    let (_g, a, _b, _out_port, _in_port, eid) = make_graph();

    let tx = GraphTransaction {
        label: None,
        ops: vec![
            GraphOp::SetNodePos {
                id: a,
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 10.0, y: 20.0 },
            },
            GraphOp::SetEdgeKind {
                id: eid,
                from: EdgeKind::Data,
                to: EdgeKind::Exec,
            },
        ],
    };

    let changes = NodeGraphChanges::from_transaction(&tx);
    assert_eq!(changes.nodes.len(), 1);
    assert_eq!(changes.edges.len(), 1);

    match &changes.nodes[0] {
        NodeChange::Position { id, position } => {
            assert_eq!(*id, a);
            assert_eq!(*position, CanvasPoint { x: 10.0, y: 20.0 });
        }
        other => panic!("unexpected node change: {other:?}"),
    }

    match &changes.edges[0] {
        EdgeChange::Kind { id, kind } => {
            assert_eq!(*id, eid);
            assert_eq!(*kind, EdgeKind::Exec);
        }
        other => panic!("unexpected edge change: {other:?}"),
    }
}

#[test]
fn changes_to_transaction_is_reversible_and_applicable() {
    let (g0, a, _b, out_port, in_port, eid) = make_graph();

    let changes = NodeGraphChanges {
        nodes: vec![NodeChange::Position {
            id: a,
            position: CanvasPoint { x: 42.0, y: 7.0 },
        }],
        edges: vec![EdgeChange::Endpoints {
            id: eid,
            from: out_port,
            to: in_port,
        }],
    };

    let tx = changes.to_transaction(&g0).expect("tx");
    let mut g1 = g0.clone();
    apply_transaction(&mut g1, &tx).expect("apply");

    assert_eq!(
        g1.nodes.get(&a).unwrap().pos,
        CanvasPoint { x: 42.0, y: 7.0 }
    );
    assert_eq!(g1.edges.get(&eid).unwrap().from, out_port);
    assert_eq!(g1.edges.get(&eid).unwrap().to, in_port);
}
