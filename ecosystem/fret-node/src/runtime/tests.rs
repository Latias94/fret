use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind, Graph, Node, NodeId, NodeKindKey, Port};
use crate::io::NodeGraphViewState;
use crate::ops::{GraphOp, GraphTransaction, apply_transaction};
use crate::runtime::apply::{apply_edge_changes, apply_node_changes};
use crate::runtime::callbacks::{
    ConnectionChange, NodeGraphCallbacks, connection_changes_from_transaction, install_callbacks,
};
use crate::runtime::changes::{EdgeChange, NodeChange, NodeGraphChanges};
use crate::runtime::events::NodeGraphStoreEvent;
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
        selectable: None,
        draggable: None,
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
            selectable: None,
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
        parent: None,
        size: None,
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
