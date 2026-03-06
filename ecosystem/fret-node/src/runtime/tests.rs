use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::runtime::apply::{apply_edge_changes, apply_node_changes};
use crate::runtime::callbacks::{
    ConnectionChange, NodeGraphCallbacks, connection_changes_from_transaction, install_callbacks,
};
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};
use crate::runtime::events::NodeGraphStoreEvent;
use crate::runtime::lookups::{ConnectionSide, NodeGraphLookups};
use crate::runtime::middleware::NodeGraphStoreMiddleware;
use crate::runtime::store::NodeGraphStore;

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
        ports: vec![out_port],
        data: serde_json::Value::Null,
    };
    let node_b = Node {
        kind: NodeKindKey::new("demo.b"),
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            selectable: None,
            deletable: None,
            reconnectable: None,
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

#[test]
fn apply_node_changes_removes_ports_and_incident_edges() {
    let (mut g0, a, b, out_port, in_port, eid) = make_graph();

    let report = apply_node_changes(&mut g0, &[NodeChange::Remove { id: a }]);
    assert!(report.did_change());
    assert_eq!(report.ignored, 0);

    assert!(!g0.nodes.contains_key(&a));
    assert!(g0.nodes.contains_key(&b));

    assert!(!g0.ports.contains_key(&out_port));
    assert!(g0.ports.contains_key(&in_port));

    assert!(!g0.edges.contains_key(&eid));
}

#[test]
fn apply_edge_changes_updates_kind_and_ignores_missing() {
    let (mut g0, _a, _b, _out_port, _in_port, eid) = make_graph();
    let missing = EdgeId::new();

    let report = apply_edge_changes(
        &mut g0,
        &[
            EdgeChange::Kind {
                id: eid,
                kind: EdgeKind::Exec,
            },
            EdgeChange::Remove { id: missing },
        ],
    );
    assert!(report.did_change());
    assert_eq!(report.ignored, 1);
    assert_eq!(g0.edges.get(&eid).unwrap().kind, EdgeKind::Exec);
}

#[test]
fn connection_changes_from_transaction_maps_edge_ops() {
    let (_g0, _a, _b, out_port, in_port, eid) = make_graph();

    let tx = GraphTransaction {
        label: None,
        ops: vec![
            GraphOp::AddEdge {
                id: eid,
                edge: Edge {
                    kind: EdgeKind::Data,
                    from: out_port,
                    to: in_port,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            },
            GraphOp::SetEdgeEndpoints {
                id: eid,
                from: crate::ops::EdgeEndpoints {
                    from: out_port,
                    to: in_port,
                },
                to: crate::ops::EdgeEndpoints {
                    from: out_port,
                    to: in_port,
                },
            },
            GraphOp::RemoveEdge {
                id: eid,
                edge: Edge {
                    kind: EdgeKind::Data,
                    from: out_port,
                    to: in_port,
                    selectable: None,
                    deletable: None,
                    reconnectable: None,
                },
            },
        ],
    };

    let changes = connection_changes_from_transaction(&tx);
    assert_eq!(changes.len(), 3);
    assert!(matches!(changes[0], ConnectionChange::Connected(_)));
    assert!(matches!(changes[1], ConnectionChange::Reconnected { .. }));
    assert!(matches!(changes[2], ConnectionChange::Disconnected(_)));
}

#[test]
fn lookups_rebuild_populates_connection_lookup() {
    let (g, a, b, out_port, in_port, eid) = make_graph();

    let mut lookups = NodeGraphLookups::default();
    lookups.rebuild_from(&g);

    assert!(lookups.node_lookup.contains_key(&a));
    assert!(lookups.node_lookup.contains_key(&b));
    assert_eq!(lookups.node_lookup.get(&a).unwrap().ports, vec![out_port]);
    assert_eq!(lookups.node_lookup.get(&b).unwrap().ports, vec![in_port]);

    assert_eq!(lookups.edge_lookup.get(&eid).unwrap().from, out_port);
    assert_eq!(lookups.edge_lookup.get(&eid).unwrap().to, in_port);

    let a_out = lookups
        .connections_for_port(a, ConnectionSide::Source, out_port)
        .expect("connections");
    assert_eq!(a_out.get(&eid).unwrap().target_node, b);

    let b_all = lookups.connections_for_node(b).expect("connections");
    assert!(b_all.contains_key(&eid));
}

#[test]
fn store_lookups_update_after_dispatch_transaction() {
    let (mut g, _a, _b, out_port, in_port, eid) = make_graph();
    g.edges.clear();

    let mut store = NodeGraphStore::new(g, NodeGraphViewState::default());
    assert!(store.lookups().edge_lookup.is_empty());

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::AddEdge {
            id: eid,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        }],
    };

    store.dispatch_transaction(&tx).expect("dispatch");
    assert!(store.lookups().edge_lookup.contains_key(&eid));
}

#[test]
fn lookups_connections_for_node_side_filters_by_direction() {
    let (g, a, b, out_port, in_port, eid) = make_graph();

    let mut lookups = NodeGraphLookups::default();
    lookups.rebuild_from(&g);

    let a_source = lookups
        .connections_for_node_side(a, ConnectionSide::Source)
        .expect("connections");
    assert!(a_source.contains_key(&eid));

    let a_target = lookups.connections_for_node_side(a, ConnectionSide::Target);
    assert!(a_target.is_none() || !a_target.unwrap().contains_key(&eid));

    let b_target = lookups
        .connections_for_node_side(b, ConnectionSide::Target)
        .expect("connections");
    assert!(b_target.contains_key(&eid));

    let b_source = lookups.connections_for_node_side(b, ConnectionSide::Source);
    assert!(b_source.is_none() || !b_source.unwrap().contains_key(&eid));

    let _ = (out_port, in_port);
}

#[test]
fn install_callbacks_receives_graph_and_view_events() {
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct Recorder {
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl NodeGraphCallbacks for Recorder {
        fn on_graph_commit(&mut self, _committed: &GraphTransaction, _changes: &NodeGraphChanges) {
            self.log.borrow_mut().push("commit");
        }

        fn on_nodes_change(&mut self, _changes: &[NodeChange]) {
            self.log.borrow_mut().push("nodes");
        }

        fn on_edges_change(&mut self, _changes: &[EdgeChange]) {
            self.log.borrow_mut().push("edges");
        }

        fn on_connection_change(&mut self, _change: ConnectionChange) {
            self.log.borrow_mut().push("conn");
        }

        fn on_view_change(&mut self, _changes: &[crate::runtime::events::ViewChange]) {
            self.log.borrow_mut().push("view");
        }
    }

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };
    let _token = install_callbacks(&mut store, recorder);

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 1.0, y: 2.0 },
        }],
    };
    let _ = store.dispatch_transaction(&tx).expect("dispatch");

    store.update_view_state(|s| {
        s.pan = CanvasPoint { x: 10.0, y: 20.0 };
    });

    let got = log.borrow().clone();
    assert!(got.contains(&"commit"));
    assert!(got.contains(&"nodes"));
    assert!(got.contains(&"view"));
}

#[test]
fn controlled_graph_can_apply_store_changes_via_callbacks() {
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone)]
    struct ControlledApply {
        graph: Rc<RefCell<Graph>>,
    }

    impl NodeGraphCallbacks for ControlledApply {
        fn on_nodes_change(&mut self, changes: &[NodeChange]) {
            apply_node_changes(&mut self.graph.borrow_mut(), changes);
        }

        fn on_edges_change(&mut self, changes: &[EdgeChange]) {
            apply_edge_changes(&mut self.graph.borrow_mut(), changes);
        }
    }

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0.clone(), NodeGraphViewState::default());

    let controlled = Rc::new(RefCell::new(g0));
    let _token = install_callbacks(
        &mut store,
        ControlledApply {
            graph: controlled.clone(),
        },
    );

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 123.0, y: 456.0 },
        }],
    };
    let _ = store.dispatch_transaction(&tx).expect("dispatch");

    let store_json = serde_json::to_value(store.graph()).expect("store json");
    let controlled_json = serde_json::to_value(&*controlled.borrow()).expect("controlled json");
    assert_eq!(store_json, controlled_json);
}

#[test]
fn install_callbacks_calls_viewport_selection_and_connection_hooks() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::core::{GroupId, PortCapacity, PortDirection, PortKind};
    use crate::ops::EdgeEndpoints;
    use crate::runtime::callbacks::SelectionChange;

    #[derive(Clone)]
    struct Recorder {
        log: Rc<RefCell<Vec<&'static str>>>,
    }

    impl NodeGraphCallbacks for Recorder {
        fn on_viewport_change(&mut self, _pan: CanvasPoint, _zoom: f32) {
            self.log.borrow_mut().push("viewport");
        }

        fn on_move(&mut self, _pan: CanvasPoint, _zoom: f32) {
            self.log.borrow_mut().push("move");
        }

        fn on_selection_change(&mut self, _sel: SelectionChange) {
            self.log.borrow_mut().push("selection");
        }

        fn on_connect(&mut self, _conn: crate::runtime::callbacks::EdgeConnection) {
            self.log.borrow_mut().push("connect");
        }

        fn on_disconnect(&mut self, _conn: crate::runtime::callbacks::EdgeConnection) {
            self.log.borrow_mut().push("disconnect");
        }

        fn on_reconnect(&mut self, _edge: EdgeId, _from: EdgeEndpoints, _to: EdgeEndpoints) {
            self.log.borrow_mut().push("reconnect");
        }

        fn on_edge_update(&mut self, _edge: EdgeId, _from: EdgeEndpoints, _to: EdgeEndpoints) {
            self.log.borrow_mut().push("edge_update");
        }
    }

    let (mut g0, a, _b, out_port, in_port, eid) = make_graph();

    let in2 = crate::core::PortId::new();
    let c = NodeId::new();
    g0.nodes.insert(
        c,
        Node {
            kind: NodeKindKey::new("demo.c"),
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
            ports: vec![in2],
            data: serde_json::Value::Null,
        },
    );
    g0.ports.insert(
        in2,
        Port {
            node: c,
            key: crate::core::PortKey::new("in2"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let log: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };
    let _token = install_callbacks(&mut store, recorder);

    store.set_viewport(CanvasPoint { x: 10.0, y: 20.0 }, 1.25);
    store.set_selection(vec![a], vec![eid], vec![GroupId::new()]);

    let e2 = EdgeId::new();
    let tx_add = GraphTransaction {
        label: None,
        ops: vec![GraphOp::AddEdge {
            id: e2,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in_port,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        }],
    };
    let _ = store.dispatch_transaction(&tx_add).expect("dispatch add");

    let tx_reconnect = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetEdgeEndpoints {
            id: e2,
            from: EdgeEndpoints {
                from: out_port,
                to: in_port,
            },
            to: EdgeEndpoints {
                from: out_port,
                to: in2,
            },
        }],
    };
    let _ = store
        .dispatch_transaction(&tx_reconnect)
        .expect("dispatch reconnect");

    let tx_remove = GraphTransaction {
        label: None,
        ops: vec![GraphOp::RemoveEdge {
            id: e2,
            edge: Edge {
                kind: EdgeKind::Data,
                from: out_port,
                to: in2,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        }],
    };
    let _ = store
        .dispatch_transaction(&tx_remove)
        .expect("dispatch remove");

    let got = log.borrow().clone();
    assert!(got.contains(&"viewport"));
    assert!(got.contains(&"move"));
    assert!(got.contains(&"selection"));
    assert!(got.contains(&"connect"));
    assert!(got.contains(&"reconnect"));
    assert!(got.contains(&"edge_update"));
    assert!(got.contains(&"disconnect"));
}

#[test]
fn install_callbacks_calls_delete_hooks_for_remove_node() {
    use std::cell::RefCell;
    use std::rc::Rc;

    use crate::ops::GraphOpBuilderExt;

    #[derive(Clone)]
    struct Recorder {
        nodes_deleted: Rc<RefCell<Vec<NodeId>>>,
        edges_deleted: Rc<RefCell<Vec<EdgeId>>>,
        disconnected: Rc<RefCell<Vec<EdgeId>>>,
    }

    impl NodeGraphCallbacks for Recorder {
        fn on_nodes_delete(&mut self, nodes: &[NodeId]) {
            self.nodes_deleted.borrow_mut().extend_from_slice(nodes);
        }

        fn on_edges_delete(&mut self, edges: &[EdgeId]) {
            self.edges_deleted.borrow_mut().extend_from_slice(edges);
        }

        fn on_disconnect(&mut self, conn: crate::runtime::callbacks::EdgeConnection) {
            self.disconnected.borrow_mut().push(conn.edge);
        }
    }

    let (g0, a, _b, _out_port, _in_port, eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let nodes_deleted: Rc<RefCell<Vec<NodeId>>> = Rc::new(RefCell::new(Vec::new()));
    let edges_deleted: Rc<RefCell<Vec<EdgeId>>> = Rc::new(RefCell::new(Vec::new()));
    let disconnected: Rc<RefCell<Vec<EdgeId>>> = Rc::new(RefCell::new(Vec::new()));

    let _token = install_callbacks(
        &mut store,
        Recorder {
            nodes_deleted: nodes_deleted.clone(),
            edges_deleted: edges_deleted.clone(),
            disconnected: disconnected.clone(),
        },
    );

    let op = store
        .graph()
        .build_remove_node_op(a)
        .expect("remove node op");
    let tx = GraphTransaction {
        label: None,
        ops: vec![op],
    };
    let _ = store.dispatch_transaction(&tx).expect("dispatch remove");

    assert!(nodes_deleted.borrow().contains(&a));
    assert!(edges_deleted.borrow().contains(&eid));
    assert!(disconnected.borrow().contains(&eid));
}

#[test]
fn store_dispatch_changes_records_history_and_supports_undo() {
    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let changes = NodeGraphChanges {
        nodes: vec![NodeChange::Position {
            id: a,
            position: CanvasPoint { x: 12.0, y: 34.0 },
        }],
        edges: Vec::new(),
    };

    let outcome = store.dispatch_changes(&changes).expect("dispatch");
    assert!(!outcome.committed.ops.is_empty());
    assert!(store.can_undo());
    assert_eq!(
        store.graph().nodes.get(&a).unwrap().pos,
        CanvasPoint { x: 12.0, y: 34.0 }
    );

    let undo = store.undo().expect("undo").expect("did undo");
    assert!(!undo.committed.ops.is_empty());
    assert_eq!(
        store.graph().nodes.get(&a).unwrap().pos,
        CanvasPoint { x: 0.0, y: 0.0 }
    );
}

#[test]
fn store_does_not_commit_rejected_profile_edits() {
    use crate::rules::{ConnectPlan, Diagnostic, DiagnosticSeverity, DiagnosticTarget};
    use crate::types::TypeDesc;

    struct RejectProfile;

    impl crate::profile::GraphProfile for RejectProfile {
        fn type_of_port(&mut self, _graph: &Graph, _port: crate::core::PortId) -> Option<TypeDesc> {
            None
        }

        fn plan_connect(
            &mut self,
            _graph: &Graph,
            _a: crate::core::PortId,
            _b: crate::core::PortId,
            _mode: crate::interaction::NodeGraphConnectionMode,
        ) -> ConnectPlan {
            ConnectPlan::reject("not used in this test")
        }

        fn validate_graph(&mut self, _graph: &Graph) -> Vec<Diagnostic> {
            vec![Diagnostic {
                key: "test.reject".to_string(),
                severity: DiagnosticSeverity::Error,
                target: DiagnosticTarget::Graph,
                message: "rejected by test profile".to_string(),
                fixes: Vec::new(),
            }]
        }
    }

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::with_profile(
        g0.clone(),
        NodeGraphViewState::default(),
        Box::new(RejectProfile),
    );

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 999.0, y: 999.0 },
        }],
    };

    let err = store.dispatch_transaction(&tx).expect_err("reject");
    let crate::runtime::store::DispatchError::Apply(crate::profile::ApplyPipelineError::Rejected {
        diagnostics,
        ..
    }) = err
    else {
        panic!("unexpected error: {err:?}");
    };
    assert!(!diagnostics.is_empty());

    assert_eq!(
        store.graph().nodes.get(&a).unwrap().pos,
        g0.nodes.get(&a).unwrap().pos
    );
    assert!(!store.can_undo());
}

#[test]
fn store_rejects_non_finite_transactions() {
    let g = Graph::new(crate::core::GraphId::from_u128(1));
    let node_id = NodeId::new();

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::AddNode {
            id: node_id,
            node: Node {
                kind: NodeKindKey::new("demo.node"),
                kind_version: 1,
                pos: CanvasPoint {
                    x: f32::NAN,
                    y: 0.0,
                },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(crate::core::CanvasSize {
                    width: 10.0,
                    height: 10.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        }],
    };

    let mut store = NodeGraphStore::new(g.clone(), NodeGraphViewState::default());
    let err = store.dispatch_transaction(&tx).expect_err("reject");
    let crate::runtime::store::DispatchError::Apply(crate::profile::ApplyPipelineError::Rejected {
        diagnostics,
        ..
    }) = err
    else {
        panic!("unexpected error: {err:?}");
    };
    assert_eq!(diagnostics[0].key, "tx.non_finite");
    assert!(store.graph().nodes.is_empty());
    assert_eq!(store.graph().graph_id, g.graph_id);
    assert!(!store.can_undo());
}

#[test]
fn store_rejects_invalid_size_transactions() {
    let g = Graph::new(crate::core::GraphId::from_u128(1));
    let node_id = NodeId::new();

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::AddNode {
            id: node_id,
            node: Node {
                kind: NodeKindKey::new("demo.node"),
                kind_version: 1,
                pos: CanvasPoint { x: 0.0, y: 0.0 },
                selectable: None,
                draggable: None,
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: Some(crate::core::CanvasSize {
                    width: 0.0,
                    height: 10.0,
                }),
                hidden: false,
                collapsed: false,
                ports: Vec::new(),
                data: serde_json::Value::Null,
            },
        }],
    };

    let mut store = NodeGraphStore::new(g.clone(), NodeGraphViewState::default());
    let err = store.dispatch_transaction(&tx).expect_err("reject");
    let crate::runtime::store::DispatchError::Apply(crate::profile::ApplyPipelineError::Rejected {
        diagnostics,
        ..
    }) = err
    else {
        panic!("unexpected error: {err:?}");
    };
    assert_eq!(diagnostics[0].key, "tx.invalid_size");
    assert!(store.graph().nodes.is_empty());
    assert_eq!(store.graph().graph_id, g.graph_id);
    assert!(!store.can_undo());
}

#[test]
fn store_middleware_can_rewrite_transactions() {
    #[derive(Debug, Default)]
    struct DropOps;

    impl NodeGraphStoreMiddleware for DropOps {
        fn before_dispatch(
            &mut self,
            _snapshot: crate::runtime::events::NodeGraphStoreSnapshot<'_>,
            tx: &mut GraphTransaction,
        ) -> Result<(), crate::profile::ApplyPipelineError> {
            tx.ops.clear();
            Ok(())
        }
    }

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default()).with_middleware(DropOps);

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 10.0, y: 20.0 },
        }],
    };

    let outcome = store.dispatch_transaction(&tx).expect("dispatch");
    assert!(outcome.committed.ops.is_empty());
    assert_eq!(
        store.graph().nodes.get(&a).unwrap().pos,
        CanvasPoint { x: 0.0, y: 0.0 }
    );
    assert!(!store.can_undo());
}

#[test]
fn store_middleware_can_reject_transactions() {
    use crate::rules::{Diagnostic, DiagnosticSeverity, DiagnosticTarget};

    #[derive(Debug, Default)]
    struct RejectAll;

    impl NodeGraphStoreMiddleware for RejectAll {
        fn before_dispatch(
            &mut self,
            _snapshot: crate::runtime::events::NodeGraphStoreSnapshot<'_>,
            _tx: &mut GraphTransaction,
        ) -> Result<(), crate::profile::ApplyPipelineError> {
            Err(crate::profile::ApplyPipelineError::Rejected {
                message: "rejected by middleware".to_string(),
                diagnostics: vec![Diagnostic {
                    key: "test.middleware.reject".to_string(),
                    severity: DiagnosticSeverity::Error,
                    target: DiagnosticTarget::Graph,
                    message: "rejected by middleware".to_string(),
                    fixes: Vec::new(),
                }],
            })
        }
    }

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store =
        NodeGraphStore::new(g0.clone(), NodeGraphViewState::default()).with_middleware(RejectAll);

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::SetNodePos {
            id: a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 10.0, y: 20.0 },
        }],
    };

    let err = store.dispatch_transaction(&tx).expect_err("reject");
    let crate::runtime::store::DispatchError::Apply(crate::profile::ApplyPipelineError::Rejected {
        diagnostics,
        ..
    }) = err
    else {
        panic!("unexpected error: {err:?}");
    };
    assert_eq!(diagnostics[0].key, "test.middleware.reject");
    assert_eq!(
        store.graph().nodes.get(&a).unwrap().pos,
        g0.nodes.get(&a).unwrap().pos
    );
    assert!(!store.can_undo());
}

#[test]
fn store_subscription_receives_graph_and_view_events_and_can_unsubscribe() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let events: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let events2 = events.clone();

    let token = store.subscribe(move |ev| match ev {
        NodeGraphStoreEvent::GraphCommitted { .. } => events2.borrow_mut().push("graph"),
        NodeGraphStoreEvent::ViewChanged { changes, .. } => {
            assert!(!changes.is_empty());
            events2.borrow_mut().push("view");
        }
    });

    store.set_viewport(crate::core::CanvasPoint { x: 1.0, y: 2.0 }, 1.25);
    store.set_selection(vec![a], Vec::new(), Vec::new());

    let changes = NodeGraphChanges {
        nodes: vec![NodeChange::Position {
            id: a,
            position: CanvasPoint { x: 5.0, y: 6.0 },
        }],
        edges: Vec::new(),
    };
    store.dispatch_changes(&changes).expect("dispatch");

    let got = events.borrow().clone();
    assert!(got.contains(&"view"));
    assert!(got.contains(&"graph"));

    assert!(store.unsubscribe(token));
    assert!(!store.unsubscribe(token));

    let before_len = events.borrow().len();
    store.set_viewport(crate::core::CanvasPoint { x: 3.0, y: 4.0 }, 2.0);
    store.dispatch_changes(&changes).expect("dispatch");
    assert_eq!(events.borrow().len(), before_len);
}

#[test]
fn store_selector_subscription_dedupes_and_tracks_graph_and_view_projections() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let node_counts: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let node_counts2 = node_counts.clone();
    store.subscribe_selector(
        |s| s.graph.nodes.len(),
        move |v| node_counts2.borrow_mut().push(*v),
    );

    let selection_counts: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(Vec::new()));
    let selection_counts2 = selection_counts.clone();
    store.subscribe_selector(
        |s| s.view_state.selected_nodes.len(),
        move |v| selection_counts2.borrow_mut().push(*v),
    );

    // Same selection twice should dedupe (no extra callback).
    store.set_selection(vec![a], Vec::new(), Vec::new());
    store.set_selection(vec![a], Vec::new(), Vec::new());

    assert_eq!(selection_counts.borrow().as_slice(), &[1]);
    assert!(node_counts.borrow().is_empty());

    // Adding a node should trigger only the node-count selector.
    let new_id = NodeId::new();
    let node = Node {
        kind: NodeKindKey::new("demo.c"),
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
        ports: Vec::new(),
        data: serde_json::Value::Null,
    };

    let tx = GraphTransaction {
        label: None,
        ops: vec![GraphOp::AddNode { id: new_id, node }],
    };
    store.dispatch_transaction(&tx).expect("dispatch");

    assert_eq!(node_counts.borrow().as_slice(), &[3]);
    assert_eq!(selection_counts.borrow().as_slice(), &[1]);
}

#[test]
fn store_selector_diff_provides_prev_and_next() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let deltas: Rc<RefCell<Vec<(usize, usize)>>> = Rc::new(RefCell::new(Vec::new()));
    let deltas2 = deltas.clone();
    store.subscribe_selector_diff(
        |s| s.view_state.selected_nodes.len(),
        move |prev, next| deltas2.borrow_mut().push((*prev, *next)),
    );

    store.set_selection(vec![a], Vec::new(), Vec::new());
    store.set_selection(vec![a], Vec::new(), Vec::new());
    store.set_selection(Vec::new(), Vec::new(), Vec::new());

    assert_eq!(deltas.borrow().as_slice(), &[(0, 1), (1, 0)]);
}

#[test]
fn store_replace_view_state_emits_view_changed_event() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, _a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let events: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let events2 = events.clone();
    store.subscribe(move |ev| match ev {
        NodeGraphStoreEvent::ViewChanged { .. } => events2.borrow_mut().push("view"),
        NodeGraphStoreEvent::GraphCommitted { .. } => events2.borrow_mut().push("graph"),
    });

    let mut vs = NodeGraphViewState::default();
    vs.pan = crate::core::CanvasPoint { x: 10.0, y: 20.0 };
    vs.zoom = 1.5;
    store.replace_view_state(vs);

    assert_eq!(events.borrow().as_slice(), &["view"]);
}

#[test]
fn store_replace_view_state_notifies_selectors_for_runtime_tuning_only_changes() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, _a, _b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let events: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let events2 = events.clone();
    store.subscribe(move |ev| match ev {
        NodeGraphStoreEvent::ViewChanged { .. } => events2.borrow_mut().push("view"),
        NodeGraphStoreEvent::GraphCommitted { .. } => events2.borrow_mut().push("graph"),
    });

    let runtime_flags: Rc<RefCell<Vec<bool>>> = Rc::new(RefCell::new(Vec::new()));
    let runtime_flags2 = runtime_flags.clone();
    store.subscribe_selector(
        |s| {
            s.view_state
                .resolved_interaction_state()
                .only_render_visible_elements
        },
        move |value| runtime_flags2.borrow_mut().push(*value),
    );

    let mut vs = store.view_state().clone();
    vs.runtime_tuning.only_render_visible_elements = false;
    store.replace_view_state(vs);

    assert!(events.borrow().is_empty());
    assert_eq!(runtime_flags.borrow().as_slice(), &[false]);
    assert!(
        !store
            .view_state()
            .runtime_tuning
            .only_render_visible_elements
    );
}

#[test]
fn store_update_view_state_notifies_selectors_for_draw_order_only_changes() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let (g0, a, b, _out_port, _in_port, _eid) = make_graph();
    let mut store = NodeGraphStore::new(g0, NodeGraphViewState::default());

    let events: Rc<RefCell<Vec<&'static str>>> = Rc::new(RefCell::new(Vec::new()));
    let events2 = events.clone();
    store.subscribe(move |ev| match ev {
        NodeGraphStoreEvent::ViewChanged { .. } => events2.borrow_mut().push("view"),
        NodeGraphStoreEvent::GraphCommitted { .. } => events2.borrow_mut().push("graph"),
    });

    let draw_order_snapshots: Rc<RefCell<Vec<Vec<NodeId>>>> = Rc::new(RefCell::new(Vec::new()));
    let draw_order_snapshots2 = draw_order_snapshots.clone();
    store.subscribe_selector(
        |s| s.view_state.draw_order.clone(),
        move |value| draw_order_snapshots2.borrow_mut().push(value.clone()),
    );

    store.update_view_state(|s| {
        s.draw_order = vec![b, a];
    });

    assert!(events.borrow().is_empty());
    assert_eq!(draw_order_snapshots.borrow().as_slice(), &[vec![b, a]]);
    assert_eq!(store.view_state().draw_order.as_slice(), &[b, a]);
}
