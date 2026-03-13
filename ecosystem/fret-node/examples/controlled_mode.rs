//! Minimal controlled-mode wiring example.
//!
//! This mirrors the ReactFlow/XyFlow mental model:
//! - the application owns the authoritative graph state,
//! - the runtime emits `NodeChange` / `EdgeChange`,
//! - the application applies changes via helpers.
//!
//! Run:
//!   cargo run -p fret-node --example controlled_mode

use std::cell::RefCell;
use std::rc::Rc;

use fret_node::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use fret_node::io::NodeGraphViewState;
use fret_node::ops::{GraphOp, GraphTransaction};
use fret_node::runtime::apply::{apply_edge_changes, apply_node_changes};
use fret_node::runtime::callbacks::{
    NodeGraphCommitCallbacks, NodeGraphGestureCallbacks, NodeGraphViewCallbacks, install_callbacks,
};
use fret_node::runtime::store::NodeGraphStore;

#[derive(Clone)]
struct ControlledApply {
    graph: Rc<RefCell<Graph>>,
}

impl NodeGraphCommitCallbacks for ControlledApply {
    fn on_nodes_change(&mut self, changes: &[fret_node::runtime::changes::NodeChange]) {
        apply_node_changes(&mut self.graph.borrow_mut(), changes);
    }

    fn on_edges_change(&mut self, changes: &[fret_node::runtime::changes::EdgeChange]) {
        apply_edge_changes(&mut self.graph.borrow_mut(), changes);
    }
}

impl NodeGraphViewCallbacks for ControlledApply {}

impl NodeGraphGestureCallbacks for ControlledApply {}

fn make_graph() -> (Graph, NodeId) {
    let graph_id = GraphId::from_u128(0x1350_0000_0000_0000_0000_0000_0000_00C0);
    let mut g = Graph::new(graph_id);

    let a = NodeId::new();
    let a_out = PortId::new();

    g.nodes.insert(
        a,
        Node {
            kind: NodeKindKey::new("example.node"),
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
            data: serde_json::Value::Null,
        },
    );
    g.ports.insert(
        a_out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let b = NodeId::new();
    let b_in = PortId::new();
    g.nodes.insert(
        b,
        Node {
            kind: NodeKindKey::new("example.node"),
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
            ports: vec![b_in],
            data: serde_json::Value::Null,
        },
    );
    g.ports.insert(
        b_in,
        Port {
            node: b,
            key: PortKey::new("in"),
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

    let e = EdgeId::new();
    g.edges.insert(
        e,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    (g, a)
}

fn main() {
    let (g0, node_a) = make_graph();
    let mut store = NodeGraphStore::new(g0.clone(), NodeGraphViewState::default());

    let app_graph = Rc::new(RefCell::new(g0));
    let _token = install_callbacks(
        &mut store,
        ControlledApply {
            graph: app_graph.clone(),
        },
    );

    let tx = GraphTransaction {
        label: Some("Move node A".to_string()),
        ops: vec![GraphOp::SetNodePos {
            id: node_a,
            from: CanvasPoint { x: 0.0, y: 0.0 },
            to: CanvasPoint { x: 42.0, y: 24.0 },
        }],
    };

    store.dispatch_transaction(&tx).expect("dispatch");

    let store_json = serde_json::to_value(store.graph()).expect("store graph json");
    let app_json = serde_json::to_value(&*app_graph.borrow()).expect("app graph json");
    assert_eq!(store_json, app_json);

    println!(
        "controlled graph stayed in sync ({} nodes)",
        store.graph().nodes.len()
    );
}
