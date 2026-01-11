use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, Group, GroupId, Node,
    NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::ops::{
    EdgeEndpoints, GraphFragment, GraphHistory, GraphOp, GraphOpBuilderExt, GraphTransaction,
    IdRemapSeed, IdRemapper, PasteTuning, apply_transaction, invert_transaction,
};
use uuid::Uuid;

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

fn make_port(node: NodeId, key: &str, dir: PortDirection) -> Port {
    Port {
        node,
        key: PortKey::new(key),
        dir,
        kind: PortKind::Data,
        capacity: PortCapacity::Multi,
        ty: None,
        data: serde_json::Value::Null,
    }
}

#[test]
fn build_remove_node_tx_captures_ports_and_edges() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    graph.nodes.get_mut(&a).unwrap().ports.push(out);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let tx = graph.build_remove_node_tx(a, "Delete Node A").expect("tx");
    assert_eq!(tx.ops.len(), 1);

    apply_transaction(&mut graph, &tx).expect("apply");

    assert!(!graph.nodes.contains_key(&a));
    assert!(!graph.ports.contains_key(&out));
    assert!(!graph.edges.contains_key(&edge_id));
}

#[test]
fn build_disconnect_port_ops_removes_incident_edges() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    graph.nodes.get_mut(&a).unwrap().ports.push(out);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let ops = graph
        .build_disconnect_port_ops(inn)
        .expect("disconnect ops");
    assert_eq!(ops.len(), 1);

    let tx = crate::ops::GraphTransaction { label: None, ops };
    apply_transaction(&mut graph, &tx).expect("apply");
    assert!(graph.edges.is_empty());
}

#[test]
fn remove_group_detaches_child_nodes_and_inverts() {
    let mut graph = Graph::default();
    let group_id = GroupId::new();
    graph.groups.insert(
        group_id,
        Group {
            title: "Group".to_string(),
            rect: crate::core::CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: crate::core::CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );

    let node_id = NodeId::new();
    let mut node = make_node("core.a");
    node.parent = Some(group_id);
    graph.nodes.insert(node_id, node);

    let tx = graph
        .build_remove_group_tx(group_id, "Delete Group")
        .expect("tx");
    apply_transaction(&mut graph, &tx).expect("apply");
    assert!(!graph.groups.contains_key(&group_id));
    assert_eq!(graph.nodes.get(&node_id).unwrap().parent, None);

    let undo = invert_transaction(&tx);
    apply_transaction(&mut graph, &undo).expect("undo apply");
    assert!(graph.groups.contains_key(&group_id));
    assert_eq!(graph.nodes.get(&node_id).unwrap().parent, Some(group_id));
}

#[test]
fn set_group_rect_and_child_positions_roundtrip_through_invert_transaction() {
    let mut graph = Graph::default();
    let group_id = GroupId::new();
    let rect0 = crate::core::CanvasRect {
        origin: CanvasPoint { x: 10.0, y: 20.0 },
        size: crate::core::CanvasSize {
            width: 100.0,
            height: 80.0,
        },
    };
    graph.groups.insert(
        group_id,
        Group {
            title: "Group".to_string(),
            rect: rect0,
            color: None,
        },
    );

    let node_a = NodeId::new();
    let node_b = NodeId::new();
    let mut a = make_node("core.a");
    a.parent = Some(group_id);
    a.pos = CanvasPoint { x: 30.0, y: 40.0 };
    let mut b = make_node("core.b");
    b.parent = Some(group_id);
    b.pos = CanvasPoint { x: 50.0, y: 60.0 };
    graph.nodes.insert(node_a, a);
    graph.nodes.insert(node_b, b);

    let rect1 = crate::core::CanvasRect {
        origin: CanvasPoint { x: 110.0, y: 120.0 },
        size: rect0.size,
    };

    let tx = GraphTransaction {
        label: Some("Move Group".to_string()),
        ops: vec![
            GraphOp::SetGroupRect {
                id: group_id,
                from: rect0,
                to: rect1,
            },
            GraphOp::SetNodePos {
                id: node_a,
                from: CanvasPoint { x: 30.0, y: 40.0 },
                to: CanvasPoint { x: 130.0, y: 140.0 },
            },
            GraphOp::SetNodePos {
                id: node_b,
                from: CanvasPoint { x: 50.0, y: 60.0 },
                to: CanvasPoint { x: 150.0, y: 160.0 },
            },
        ],
    };

    apply_transaction(&mut graph, &tx).expect("apply");
    assert_eq!(graph.groups.get(&group_id).unwrap().rect, rect1);
    assert_eq!(
        graph.nodes.get(&node_a).unwrap().pos,
        CanvasPoint { x: 130.0, y: 140.0 }
    );
    assert_eq!(
        graph.nodes.get(&node_b).unwrap().pos,
        CanvasPoint { x: 150.0, y: 160.0 }
    );

    let undo = invert_transaction(&tx);
    apply_transaction(&mut graph, &undo).expect("undo apply");
    assert_eq!(graph.groups.get(&group_id).unwrap().rect, rect0);
    assert_eq!(
        graph.nodes.get(&node_a).unwrap().pos,
        CanvasPoint { x: 30.0, y: 40.0 }
    );
    assert_eq!(
        graph.nodes.get(&node_b).unwrap().pos,
        CanvasPoint { x: 50.0, y: 60.0 }
    );
}

#[test]
fn set_node_size_roundtrips_through_invert_transaction() {
    let mut graph = Graph::default();
    let node_id = NodeId::new();
    graph.nodes.insert(node_id, make_node("core.a"));

    let tx = GraphTransaction {
        label: Some("Resize".to_string()),
        ops: vec![GraphOp::SetNodeSize {
            id: node_id,
            from: None,
            to: Some(crate::core::CanvasSize {
                width: 333.0,
                height: 222.0,
            }),
        }],
    };

    apply_transaction(&mut graph, &tx).expect("apply");
    assert_eq!(
        graph.nodes.get(&node_id).unwrap().size,
        Some(crate::core::CanvasSize {
            width: 333.0,
            height: 222.0,
        })
    );

    let undo = invert_transaction(&tx);
    apply_transaction(&mut graph, &undo).expect("undo apply");
    assert_eq!(graph.nodes.get(&node_id).unwrap().size, None);
}

#[test]
fn set_group_title_roundtrips_through_invert_transaction() {
    let mut graph = Graph::default();
    let group_id = GroupId::new();
    graph.groups.insert(
        group_id,
        Group {
            title: "Group".to_string(),
            rect: crate::core::CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: crate::core::CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );

    let tx = GraphTransaction {
        label: Some("Rename Group".to_string()),
        ops: vec![GraphOp::SetGroupTitle {
            id: group_id,
            from: "Group".to_string(),
            to: "My Group".to_string(),
        }],
    };

    apply_transaction(&mut graph, &tx).expect("apply");
    assert_eq!(graph.groups.get(&group_id).unwrap().title, "My Group");

    let undo = invert_transaction(&tx);
    apply_transaction(&mut graph, &undo).expect("undo apply");
    assert_eq!(graph.groups.get(&group_id).unwrap().title, "Group");
}

#[test]
fn set_node_data_roundtrips_through_invert_transaction() {
    let mut graph = Graph::default();
    let node = NodeId::new();
    graph.nodes.insert(node, make_node("demo.const"));

    let tx = GraphTransaction {
        label: Some("Set value".to_string()),
        ops: vec![GraphOp::SetNodeData {
            id: node,
            from: serde_json::Value::Null,
            to: serde_json::json!({ "value": 1.25 }),
        }],
    };

    apply_transaction(&mut graph, &tx).expect("apply");
    assert_eq!(
        graph.nodes.get(&node).unwrap().data,
        serde_json::json!({ "value": 1.25 })
    );

    let inverse = invert_transaction(&tx);
    apply_transaction(&mut graph, &inverse).expect("apply inverse");
    assert_eq!(
        graph.nodes.get(&node).unwrap().data,
        serde_json::Value::Null
    );
}

#[test]
fn set_edge_endpoints_updates_edge_in_place() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));
    graph.nodes.insert(c, make_node("core.c"));

    let out1 = PortId::new();
    let out2 = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out1, make_port(a, "out1", PortDirection::Out));
    graph
        .ports
        .insert(out2, make_port(c, "out2", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));

    graph.nodes.get_mut(&a).unwrap().ports.push(out1);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);
    graph.nodes.get_mut(&c).unwrap().ports.push(out2);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out1,
            to: inn,
            selectable: None,
        },
    );

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetEdgeEndpoints {
            id: edge_id,
            from: EdgeEndpoints {
                from: out1,
                to: inn,
            },
            to: EdgeEndpoints {
                from: out2,
                to: inn,
            },
        }],
    };
    apply_transaction(&mut graph, &tx).expect("apply");

    let edge = graph.edges.get(&edge_id).expect("edge");
    assert_eq!(edge.from, out2);
    assert_eq!(edge.to, inn);
}

#[test]
fn fragment_paste_transaction_is_deterministic_for_seed() {
    let mut graph = Graph::default();
    let group_id = GroupId::new();
    graph.groups.insert(
        group_id,
        Group {
            title: "G".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );
    let a = NodeId::new();
    let b = NodeId::new();
    let mut na = make_node("core.a");
    na.parent = Some(group_id);
    let mut nb = make_node("core.b");
    nb.parent = Some(group_id);
    graph.nodes.insert(a, na);
    graph.nodes.insert(b, nb);

    let out = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    graph.nodes.get_mut(&a).unwrap().ports.push(out);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let fragment = GraphFragment::from_selection(&graph, [a, b], [group_id]);
    let remapper = IdRemapper::new(IdRemapSeed(Uuid::nil()));
    let tuning = PasteTuning {
        offset: CanvasPoint { x: 10.0, y: 20.0 },
    };

    let tx1 = fragment.to_paste_transaction(&remapper, tuning);
    let tx2 = fragment.to_paste_transaction(&remapper, tuning);

    // Deterministic for a given seed and input.
    assert_eq!(
        serde_json::to_string(&tx1.ops).unwrap(),
        serde_json::to_string(&tx2.ops).unwrap()
    );

    // Apply into a new graph succeeds and preserves counts.
    let mut dst = Graph::default();
    apply_transaction(&mut dst, &tx1).unwrap();
    assert_eq!(dst.nodes.len(), 2);
    assert_eq!(dst.ports.len(), 2);
    assert_eq!(dst.edges.len(), 1);
    assert_eq!(dst.groups.len(), 1);
}

#[test]
fn invert_transaction_restores_graph_state() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    graph.nodes.get_mut(&a).unwrap().ports.push(out);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let baseline = serde_json::to_value(&graph).unwrap();

    let tx = graph.build_remove_node_tx(a, "Delete Node A").expect("tx");
    apply_transaction(&mut graph, &tx).expect("apply forward");

    let inverse = invert_transaction(&tx);
    apply_transaction(&mut graph, &inverse).expect("apply inverse");

    let restored = serde_json::to_value(&graph).unwrap();
    assert_eq!(restored, baseline);
}

#[test]
fn history_undo_redo_roundtrip() {
    let mut graph = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    graph.nodes.insert(a, make_node("core.a"));
    graph.nodes.insert(b, make_node("core.b"));

    let out = PortId::new();
    let inn = PortId::new();
    graph
        .ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    graph
        .ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    graph.nodes.get_mut(&a).unwrap().ports.push(out);
    graph.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::new();
    graph.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
        },
    );

    let baseline = serde_json::to_value(&graph).unwrap();

    let tx = graph.build_remove_node_tx(a, "Delete Node A").expect("tx");
    apply_transaction(&mut graph, &tx).expect("apply forward");
    let forward_state = serde_json::to_value(&graph).unwrap();

    let mut history = GraphHistory::default();
    history.record(tx.clone());

    history
        .undo(|undo_tx| {
            apply_transaction(&mut graph, undo_tx).expect("apply undo");
            Ok::<GraphTransaction, ()>(undo_tx.clone())
        })
        .unwrap();
    assert_eq!(serde_json::to_value(&graph).unwrap(), baseline);

    history
        .redo(|redo_tx| {
            apply_transaction(&mut graph, redo_tx).expect("apply redo");
            Ok::<GraphTransaction, ()>(redo_tx.clone())
        })
        .unwrap();
    assert_eq!(serde_json::to_value(&graph).unwrap(), forward_state);
}
