use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, GraphImport,
    Group, GroupId, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey,
    PortKind, StickyNote, StickyNoteId, Symbol, SymbolId,
};
use crate::ops::{
    EdgeEndpoints, GraphFragment, GraphHistory, GraphOp, GraphOpBuilderExt, GraphTransaction,
    IdRemapSeed, IdRemapper, PasteTuning, apply_transaction, graph_diff, invert_transaction,
};
use crate::types::TypeDesc;
use uuid::Uuid;

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
        hidden: false,
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
        connectable: None,
        connectable_start: None,
        connectable_end: None,
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
            deletable: None,
            reconnectable: None,
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
            deletable: None,
            reconnectable: None,
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
fn normalize_transaction_drops_noop_set_ops() {
    let node_id = NodeId::new();
    let p0 = CanvasPoint { x: 10.0, y: 20.0 };

    let tx = GraphTransaction {
        label: Some("Normalize".to_string()),
        ops: vec![
            GraphOp::SetNodePos {
                id: node_id,
                from: p0,
                to: p0,
            },
            GraphOp::SetNodeCollapsed {
                id: node_id,
                from: false,
                to: false,
            },
            GraphOp::SetNodeData {
                id: node_id,
                from: serde_json::Value::Null,
                to: serde_json::Value::Null,
            },
            GraphOp::AddNode {
                id: node_id,
                node: make_node("core.a"),
            },
        ],
    };

    let normalized = crate::ops::normalize_transaction(tx);
    assert_eq!(normalized.label.as_deref(), Some("Normalize"));
    assert_eq!(normalized.ops.len(), 1);
    assert!(matches!(normalized.ops[0], GraphOp::AddNode { .. }));
}

#[test]
fn normalize_transaction_keeps_non_noop_set_ops() {
    let node_id = NodeId::new();

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: node_id,
            from: CanvasPoint { x: 10.0, y: 20.0 },
            to: CanvasPoint { x: 11.0, y: 21.0 },
        }],
    };

    let normalized = crate::ops::normalize_transaction(tx);
    assert_eq!(normalized.ops.len(), 1);
    assert!(matches!(normalized.ops[0], GraphOp::SetNodePos { .. }));
}

#[test]
fn normalize_transaction_coalesces_setter_chains_and_drops_resulting_noops() {
    let node_id = NodeId::new();

    let tx = GraphTransaction {
        label: None,
        ops: vec![
            GraphOp::SetNodePos {
                id: node_id,
                from: CanvasPoint { x: 0.0, y: 0.0 },
                to: CanvasPoint { x: 10.0, y: 20.0 },
            },
            GraphOp::SetNodePos {
                id: node_id,
                from: CanvasPoint { x: 10.0, y: 20.0 },
                to: CanvasPoint { x: 0.0, y: 0.0 },
            },
        ],
    };

    let normalized = crate::ops::normalize_transaction(tx);
    assert!(normalized.ops.is_empty());
}

#[test]
fn normalize_transaction_coalesces_setter_chains_when_chained() {
    let node_id = NodeId::new();

    let tx = GraphTransaction {
        label: None,
        ops: vec![
            GraphOp::SetNodeCollapsed {
                id: node_id,
                from: false,
                to: true,
            },
            GraphOp::SetNodeCollapsed {
                id: node_id,
                from: true,
                to: false,
            },
            GraphOp::SetNodeCollapsed {
                id: node_id,
                from: false,
                to: true,
            },
        ],
    };

    let normalized = crate::ops::normalize_transaction(tx);
    assert_eq!(normalized.ops.len(), 1);
    assert!(matches!(
        &normalized.ops[0],
        GraphOp::SetNodeCollapsed {
            id,
            from: false,
            to: true
        } if *id == node_id
    ));
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
            deletable: None,
            reconnectable: None,
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
            deletable: None,
            reconnectable: None,
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
            deletable: None,
            reconnectable: None,
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
            deletable: None,
            reconnectable: None,
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

#[test]
fn symbol_setters_roundtrip_through_normalize_and_invert() {
    let mut graph = Graph::default();
    let symbol_id = SymbolId::new();
    graph.symbols.insert(
        symbol_id,
        Symbol {
            name: "A".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );

    let baseline = serde_json::to_value(&graph).unwrap();

    let mut tx = GraphTransaction::new();
    tx.ops.push(GraphOp::SetSymbolName {
        id: symbol_id,
        from: "A".to_string(),
        to: "B".to_string(),
    });
    tx.ops.push(GraphOp::SetSymbolName {
        id: symbol_id,
        from: "B".to_string(),
        to: "C".to_string(),
    });
    tx.ops.push(GraphOp::SetSymbolType {
        id: symbol_id,
        from: None,
        to: Some(TypeDesc::Int),
    });
    tx.ops.push(GraphOp::SetSymbolDefaultValue {
        id: symbol_id,
        from: None,
        to: Some(serde_json::json!(123)),
    });

    let tx = crate::ops::normalize_transaction(tx);
    assert!(
        tx.ops.len() < 4,
        "expected normalize to coalesce setter chain"
    );

    apply_transaction(&mut graph, &tx).expect("apply forward");
    assert_eq!(graph.symbols.get(&symbol_id).unwrap().name, "C");
    assert_eq!(
        graph.symbols.get(&symbol_id).unwrap().ty,
        Some(TypeDesc::Int)
    );
    assert_eq!(
        graph.symbols.get(&symbol_id).unwrap().default_value,
        Some(serde_json::json!(123))
    );

    let inverse = invert_transaction(&tx);
    apply_transaction(&mut graph, &inverse).expect("apply inverse");
    assert_eq!(serde_json::to_value(&graph).unwrap(), baseline);
}

#[test]
fn graph_import_ops_roundtrip_through_normalize_and_invert() {
    let mut graph = Graph::default();
    let baseline = serde_json::to_value(&graph).unwrap();

    let imported = GraphId::from_u128(2);
    let import = GraphImport {
        alias: Some("math".to_string()),
    };

    let mut tx = GraphTransaction::new();
    tx.ops.push(GraphOp::AddImport {
        id: imported,
        import: import.clone(),
    });
    tx.ops.push(GraphOp::SetImportAlias {
        id: imported,
        from: Some("math".to_string()),
        to: Some("stdlib".to_string()),
    });
    tx.ops.push(GraphOp::SetImportAlias {
        id: imported,
        from: Some("stdlib".to_string()),
        to: None,
    });

    let tx = crate::ops::normalize_transaction(tx);
    apply_transaction(&mut graph, &tx).expect("apply");
    assert!(graph.imports.contains_key(&imported));
    assert_eq!(graph.imports.get(&imported).unwrap().alias, None);

    let inverse = invert_transaction(&tx);
    apply_transaction(&mut graph, &inverse).expect("apply inverse");
    assert_eq!(serde_json::to_value(&graph).unwrap(), baseline);
}

#[test]
fn graph_diff_is_deterministic_and_roundtrips() {
    let mut from = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    from.nodes.insert(a, make_node("core.a"));
    from.nodes.insert(b, make_node("core.b"));

    let group_id = GroupId::new();
    from.groups.insert(
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
    from.nodes.get_mut(&a).unwrap().parent = Some(group_id);

    let out = PortId::from_u128(10);
    let inn = PortId::from_u128(11);
    from.ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    from.ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    from.nodes.get_mut(&a).unwrap().ports.push(out);
    from.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::from_u128(123);
    from.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let imported = GraphId::from_u128(10);
    from.imports.insert(imported, GraphImport::default());

    let symbol_id = SymbolId::from_u128(1);
    from.symbols.insert(
        symbol_id,
        Symbol {
            name: "S".to_string(),
            ty: None,
            default_value: None,
            meta: serde_json::Value::Null,
        },
    );

    let note_id = StickyNoteId::new();
    from.sticky_notes.insert(
        note_id,
        StickyNote {
            text: "N".to_string(),
            rect: CanvasRect {
                origin: CanvasPoint { x: 5.0, y: 6.0 },
                size: CanvasSize {
                    width: 7.0,
                    height: 8.0,
                },
            },
            color: None,
        },
    );

    let mut to = from.clone();
    to.imports.insert(
        imported,
        GraphImport {
            alias: Some("stdlib".to_string()),
        },
    );
    to.symbols.insert(
        symbol_id,
        Symbol {
            name: "T".to_string(),
            ty: Some(TypeDesc::Int),
            default_value: Some(serde_json::json!(123)),
            meta: serde_json::json!({ "k": 1 }),
        },
    );
    if let Some(group) = to.groups.get_mut(&group_id) {
        group.color = Some("red".to_string());
    }
    if let Some(edge) = to.edges.get_mut(&edge_id) {
        edge.deletable = Some(true);
        edge.reconnectable = Some(crate::core::EdgeReconnectable::Endpoint(
            crate::core::EdgeReconnectableEndpoint::Target,
        ));
    }
    if let Some(port) = to.ports.get_mut(&out) {
        port.connectable = Some(false);
        port.ty = Some(TypeDesc::String);
        port.data = serde_json::json!({ "p": 1 });
    }
    if let Some(node) = to.nodes.get_mut(&a) {
        node.pos.x = 42.0;
        node.selectable = Some(false);
        node.draggable = Some(false);
        node.connectable = Some(false);
        node.deletable = Some(false);
        node.extent = Some(crate::core::NodeExtent::Rect {
            rect: CanvasRect {
                origin: CanvasPoint { x: 1.0, y: 2.0 },
                size: CanvasSize {
                    width: 3.0,
                    height: 4.0,
                },
            },
        });
        node.expand_parent = Some(true);
        node.hidden = true;
    }
    if let Some(note) = to.sticky_notes.get_mut(&note_id) {
        note.text = "M".to_string();
        note.rect.origin.x = 9.0;
        note.color = Some("yellow".to_string());
    }

    let tx1 = graph_diff(&from, &to);
    let tx2 = graph_diff(&from, &to);
    assert_eq!(
        serde_json::to_string(&tx1.ops).unwrap(),
        serde_json::to_string(&tx2.ops).unwrap(),
        "diff must be deterministic"
    );

    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetNodeSelectable { id, .. } if *id == a)),
        "diff must include node setter ops for changed fields"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetNodeExtent { id, .. } if *id == a)),
        "diff must include node setter ops for changed fields"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetNodeHidden { id, .. } if *id == a)),
        "diff must include node setter ops for changed fields"
    );
    assert!(
        !tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveNode { id, .. } if *id == a)),
        "diff must not use destructive node removal for soft field changes"
    );

    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetGroupColor { id, .. } if *id == group_id)),
        "diff must prefer group setter ops over remove+add to preserve parent bindings"
    );
    assert!(
        !tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveGroup { id, .. } if *id == group_id)),
        "diff must not remove groups for color-only changes"
    );

    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetStickyNoteText { id, .. } if *id == note_id)),
        "diff must use sticky note setter ops for field changes"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetStickyNoteRect { id, .. } if *id == note_id)),
        "diff must use sticky note setter ops for field changes"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetStickyNoteColor { id, .. } if *id == note_id)),
        "diff must use sticky note setter ops for field changes"
    );
    assert!(
        !tx1.ops.iter().any(|op| {
            matches!(op, GraphOp::RemoveStickyNote { id, .. } if *id == note_id)
                || matches!(op, GraphOp::AddStickyNote { id, .. } if *id == note_id)
        }),
        "diff must not fall back to remove+add for sticky note field changes"
    );

    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetPortConnectable { id, .. } if *id == out)),
        "diff must use port setter ops for soft fields"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetPortType { id, .. } if *id == out)),
        "diff must use port setter ops for soft fields"
    );
    assert!(
        tx1.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetPortData { id, .. } if *id == out)),
        "diff must use port setter ops for soft fields"
    );
    assert!(
        !tx1.ops.iter().any(|op| {
            matches!(op, GraphOp::RemovePort { id, .. } if *id == out)
                || matches!(op, GraphOp::AddPort { id, .. } if *id == out)
        }),
        "diff must not fall back to remove+add for soft port changes"
    );

    let mut patched = from.clone();
    apply_transaction(&mut patched, &tx1).expect("apply diff");
    assert_eq!(
        serde_json::to_value(&patched).unwrap(),
        serde_json::to_value(&to).unwrap(),
        "diff must roundtrip"
    );
}

#[test]
fn graph_diff_roundtrips_when_deleting_a_node_with_ports_and_edges() {
    let mut from = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    from.nodes.insert(a, make_node("core.a"));
    from.nodes.insert(b, make_node("core.b"));

    let out = PortId::from_u128(20);
    let inn = PortId::from_u128(21);
    from.ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    from.ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    from.nodes.get_mut(&a).unwrap().ports.push(out);
    from.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::from_u128(456);
    from.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut to = from.clone();
    to.nodes.remove(&b);
    to.ports.remove(&inn);
    to.edges.remove(&edge_id);

    let tx = graph_diff(&from, &to);
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveNode { id, .. } if *id == b)),
        "diff must use reversible RemoveNode for node deletion"
    );
    assert!(
        !tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemovePort { id, .. } if *id == inn)),
        "diff must not double-remove ports that are already removed by RemoveNode"
    );
    assert!(
        !tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveEdge { id, .. } if *id == edge_id)),
        "diff must not double-remove edges that are already removed by RemoveNode"
    );

    let mut patched = from.clone();
    apply_transaction(&mut patched, &tx).expect("apply diff");
    assert_eq!(
        serde_json::to_value(&patched).unwrap(),
        serde_json::to_value(&to).unwrap(),
        "diff must roundtrip"
    );
}

#[test]
fn graph_diff_roundtrips_when_deleting_a_port_with_incident_edges() {
    let mut from = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    from.nodes.insert(a, make_node("core.a"));
    from.nodes.insert(b, make_node("core.b"));

    let out = PortId::from_u128(30);
    let inn = PortId::from_u128(31);
    from.ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    from.ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    from.nodes.get_mut(&a).unwrap().ports.push(out);
    from.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::from_u128(789);
    from.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut to = from.clone();
    to.nodes.get_mut(&a).unwrap().ports.retain(|p| *p != out);
    to.ports.remove(&out);
    to.edges.remove(&edge_id);

    let tx = graph_diff(&from, &to);
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemovePort { id, .. } if *id == out)),
        "diff must use reversible RemovePort for port deletion"
    );
    assert!(
        !tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveEdge { id, .. } if *id == edge_id)),
        "diff must not double-remove edges that are already removed by RemovePort"
    );

    let mut patched = from.clone();
    apply_transaction(&mut patched, &tx).expect("apply diff");
    assert_eq!(
        serde_json::to_value(&patched).unwrap(),
        serde_json::to_value(&to).unwrap(),
        "diff must roundtrip"
    );
}

#[test]
fn graph_diff_roundtrips_when_a_port_changes_structurally() {
    let mut from = Graph::default();
    let a = NodeId::new();
    let b = NodeId::new();
    from.nodes.insert(a, make_node("core.a"));
    from.nodes.insert(b, make_node("core.b"));

    let out = PortId::from_u128(40);
    let inn = PortId::from_u128(41);
    from.ports
        .insert(out, make_port(a, "out", PortDirection::Out));
    from.ports
        .insert(inn, make_port(b, "in", PortDirection::In));
    from.nodes.get_mut(&a).unwrap().ports.push(out);
    from.nodes.get_mut(&b).unwrap().ports.push(inn);

    let edge_id = EdgeId::from_u128(1010);
    from.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut to = from.clone();
    to.ports.get_mut(&out).unwrap().key = PortKey::new("out2");

    let tx = graph_diff(&from, &to);
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemovePort { id, .. } if *id == out)),
        "diff must represent structural port changes as remove+add"
    );
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::AddPort { id, .. } if *id == out)),
        "diff must represent structural port changes as remove+add"
    );
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::SetNodePorts { id, .. } if *id == a)),
        "diff must restore node port ordering after remove+add"
    );
    assert!(
        tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::AddEdge { id, .. } if *id == edge_id)),
        "diff must re-add incident edges removed by RemovePort when they still exist in 'to'"
    );
    assert!(
        !tx.ops
            .iter()
            .any(|op| matches!(op, GraphOp::RemoveEdge { id, .. } if *id == edge_id)),
        "diff must not double-remove edges that are removed by RemovePort"
    );

    let mut patched = from.clone();
    apply_transaction(&mut patched, &tx).expect("apply diff");
    assert_eq!(
        serde_json::to_value(&patched).unwrap(),
        serde_json::to_value(&to).unwrap(),
        "diff must roundtrip"
    );
}
