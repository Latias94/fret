use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use fret_core::{Event, Modifiers, MouseButton, Point, PointerEvent, PointerType, Px, Rect, Size};
use fret_ui::retained_bridge::Widget as _;

use crate::core::Graph;
use crate::rules::EdgeEndpoint;
use crate::runtime::callbacks::{
    ConnectEnd, ConnectStart, NodeDragEnd, NodeDragStart, NodeGraphCallbacks, ViewportMoveEnd,
    ViewportMoveStart,
};

use super::super::NodeGraphCanvas;
use super::{
    NullServices, TestUiHostImpl, event_cx, insert_graph_view, make_host_graph_view,
    make_test_graph_two_nodes_with_ports_spaced_x,
};
use crate::ui::canvas::state::{
    NodeDrag, PendingNodeDrag, PendingNodeSelectAction, PendingWireDrag, WireDrag, WireDragKind,
};

#[derive(Clone)]
struct Recorder {
    log: Rc<RefCell<Vec<String>>>,
}

impl NodeGraphCallbacks for Recorder {
    fn on_connect_start(&mut self, ev: ConnectStart) {
        self.log.borrow_mut().push(format!("start:{:?}", ev.kind));
    }

    fn on_connect_end(&mut self, ev: ConnectEnd) {
        self.log
            .borrow_mut()
            .push(format!("end:{:?}:{:?}", ev.outcome, ev.target));
    }

    fn on_reconnect_start(&mut self, ev: ConnectStart) {
        self.log
            .borrow_mut()
            .push(format!("reconnect_start:{:?}", ev.kind));
    }

    fn on_reconnect_end(&mut self, ev: ConnectEnd) {
        self.log
            .borrow_mut()
            .push(format!("reconnect_end:{:?}:{:?}", ev.outcome, ev.target));
    }

    fn on_edge_update_start(&mut self, ev: ConnectStart) {
        self.log
            .borrow_mut()
            .push(format!("edge_update_start:{:?}", ev.kind));
    }

    fn on_edge_update_end(&mut self, ev: ConnectEnd) {
        self.log
            .borrow_mut()
            .push(format!("edge_update_end:{:?}:{:?}", ev.outcome, ev.target));
    }

    fn on_move_start(&mut self, ev: ViewportMoveStart) {
        self.log
            .borrow_mut()
            .push(format!("move_start:{:?}", ev.kind));
    }

    fn on_move_end(&mut self, ev: ViewportMoveEnd) {
        self.log
            .borrow_mut()
            .push(format!("move_end:{:?}:{:?}", ev.kind, ev.outcome));
    }

    fn on_node_drag_start(&mut self, ev: NodeDragStart) {
        self.log
            .borrow_mut()
            .push(format!("node_drag_start:{:?}", ev.primary));
    }

    fn on_node_drag_end(&mut self, ev: NodeDragEnd) {
        self.log
            .borrow_mut()
            .push(format!("node_drag_end:{:?}:{:?}", ev.primary, ev.outcome));
    }

    fn on_node_drag(&mut self, primary: crate::core::NodeId, nodes: &[crate::core::NodeId]) {
        self.log
            .borrow_mut()
            .push(format!("node_drag:{:?}:{}", primary, nodes.len()));
    }
}

fn make_bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

#[test]
fn click_connect_emits_connect_start_and_committed_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connect_on_click = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(40.0), Px(40.0));
    canvas.interaction.pending_wire_drag = Some(PendingWireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        start_pos: pos,
    });

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(b_in),
        )
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("start:")));
    assert!(!got.iter().any(|s| s.starts_with("edge_update_start:")));
    assert!(
        got.iter()
            .any(|s| s.contains("end:Committed") && s.contains("Some"))
    );
}

#[test]
fn escape_cancel_emits_connect_end_canceled() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.connect_on_click = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(40.0), Px(40.0));
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos,
    });

    super::super::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("end:Canceled")));
    let _ = snapshot;
}

#[test]
fn rejected_drop_emits_connect_end_rejected() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    graph_value.ports.get_mut(&b_in).unwrap().kind = crate::core::PortKind::Exec;
    graph_value.nodes.get_mut(&b).unwrap().ports = vec![b_in];

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::New {
            from: a_out,
            bundle: Vec::new(),
        },
        pos: Point::new(Px(40.0), Px(40.0)),
    });

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(b_in),
        )
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("end:Rejected")));
}

#[test]
fn reconnect_emits_reconnect_start_and_committed_end() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let c = crate::core::NodeId::new();
    let c_in = crate::core::PortId::new();
    graph_value.nodes.insert(
        c,
        crate::core::Node {
            kind: crate::core::NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 280.0, y: 160.0 },
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
            ports: vec![c_in],
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        c_in,
        crate::core::Port {
            node: c,
            key: crate::core::PortKey::new("in2"),
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

    let edge = crate::core::EdgeId::new();
    graph_value.edges.insert(
        edge,
        crate::core::Edge {
            kind: crate::core::EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let kind = WireDragKind::Reconnect {
        edge,
        endpoint: EdgeEndpoint::To,
        fixed: a_out,
    };
    canvas.emit_connect_start(&snapshot, &kind);
    canvas.interaction.wire_drag = Some(WireDrag {
        kind,
        pos: Point::new(Px(40.0), Px(40.0)),
    });

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(c_in),
        )
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("reconnect_start:")));
    assert!(got.iter().any(|s| s.starts_with("edge_update_start:")));
    assert!(
        got.iter()
            .any(|s| s.contains("reconnect_end:Committed") && s.contains("Some"))
    );
    assert!(
        got.iter()
            .any(|s| s.contains("edge_update_end:Committed") && s.contains("Some"))
    );
}

#[test]
fn reconnect_escape_cancel_emits_reconnect_end_canceled() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let edge = crate::core::EdgeId::new();
    graph_value.edges.insert(
        edge,
        crate::core::Edge {
            kind: crate::core::EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);

    canvas.interaction.wire_drag = Some(WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::To,
            fixed: a_out,
        },
        pos: Point::new(Px(40.0), Px(40.0)),
    });

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    super::super::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("reconnect_end:Canceled")));
    assert!(got.iter().any(|s| s.contains("edge_update_end:Canceled")));
}

#[test]
fn panning_emits_move_start_and_move_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(100.0), Px(100.0));
    assert!(super::super::pan_zoom::begin_panning(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        MouseButton::Middle,
    ));
    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        MouseButton::Middle,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("move_start:")));
    assert!(got.iter().any(|s| s.contains("move_end:PanDrag:Ended")));
}

#[test]
fn escape_cancel_panning_emits_move_end_canceled() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(100.0), Px(100.0));
    assert!(super::super::pan_zoom::begin_panning(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        MouseButton::Middle,
    ));

    super::super::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.contains("move_end:PanDrag:Canceled")));
}

#[test]
fn node_drag_start_and_escape_cancel_emits_node_drag_end_canceled() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_draggable = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(100.0), Px(100.0));
    canvas.interaction.pending_node_drag = Some(PendingNodeDrag {
        primary: a,
        nodes: vec![a],
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: start,
        select_action: PendingNodeSelectAction::None,
        drag_enabled: true,
    });

    let _ = super::super::pending_drag::handle_pending_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(400.0), Px(400.0)),
        snapshot.zoom,
    );

    super::super::cancel::handle_escape_cancel(&mut canvas, &mut cx);

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("node_drag_start:")));
    assert!(
        got.iter()
            .any(|s| s.contains("node_drag_end") && s.contains("Canceled"))
    );
}

#[test]
fn pan_inertia_emits_move_end_after_inertia_stops() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_inertia.enabled = true;
        s.interaction.pan_inertia.decay_per_s = 200.0;
        s.interaction.pan_inertia.min_speed = 1.0;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(100.0), Px(100.0));
    assert!(super::super::pan_zoom::begin_panning(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        MouseButton::Middle,
    ));
    assert!(super::super::pan_zoom::handle_panning_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(200.0), Px(200.0)),
    ));
    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        MouseButton::Middle,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_start:PanDrag"));
    assert!(got.iter().any(|s| s == "move_end:PanDrag:Ended"));
    assert!(got.iter().any(|s| s == "move_start:PanInertia"));
    assert!(
        !got.iter().any(|s| s == "move_end:PanInertia:Ended"),
        "move_end should be deferred while inertia is active"
    );

    let token = canvas
        .interaction
        .pan_inertia
        .as_ref()
        .expect("pan inertia started")
        .timer;
    canvas
        .interaction
        .pan_inertia
        .as_mut()
        .expect("pan inertia started")
        .last_tick_at = Instant::now() - Duration::from_millis(200);

    canvas.event(&mut cx, &Event::Timer { token });

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_end:PanInertia:Ended"));
}

#[test]
fn node_drag_pointer_up_emits_node_drag_end_committed() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let start_pos = graph_value.nodes.get(&a).unwrap().pos;
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: a,
        node_ids: vec![a],
        nodes: vec![(a, start_pos)],
        current_nodes: vec![(a, start_pos)],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(100.0), Px(100.0)),
    });

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(super::super::node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(110.0), Px(110.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(110.0), Px(110.0)),
        MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let got = log.borrow().clone();
    assert!(
        got.iter()
            .any(|s| s.contains("node_drag_end") && s.contains("Committed"))
    );
}

#[test]
fn node_drag_move_emits_on_node_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let start_pos = graph_value.nodes.get(&a).unwrap().pos;
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: a,
        node_ids: vec![a],
        nodes: vec![(a, start_pos)],
        current_nodes: vec![(a, start_pos)],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(super::super::node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(10.0), Px(10.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s.starts_with("node_drag:")));
}

#[test]
fn wheel_zoom_emits_move_start_and_debounced_move_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_scroll = true;
        s.interaction.zoom_on_scroll_speed = 1.0;
        s.interaction.zoom_activation_key = crate::io::NodeGraphZoomActivationKey::None;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(100.0), Px(100.0));
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            delta: Point::new(Px(0.0), Px(-120.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_start:ZoomWheel"));

    let token = canvas
        .interaction
        .viewport_move_debounce
        .as_ref()
        .expect("debounce timer")
        .timer;

    canvas.event(&mut cx, &Event::Timer { token });

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_end:ZoomWheel:Ended"));
    let _ = snapshot;
}

#[test]
fn pinch_zoom_emits_move_start_and_debounced_move_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_pinch = true;
        s.interaction.zoom_on_pinch_speed = 1.0;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(100.0), Px(100.0));
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::PinchGesture {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            delta: 1.0,
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_start:ZoomPinch"));

    let token = canvas
        .interaction
        .viewport_move_debounce
        .as_ref()
        .expect("debounce timer")
        .timer;
    canvas.event(&mut cx, &Event::Timer { token });

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_end:ZoomPinch:Ended"));
    let _ = snapshot;
}

#[test]
fn wheel_pan_emits_move_start_and_debounced_move_end() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let (graph, view) = insert_graph_view(&mut host, graph_value);

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_scroll = false;
        s.interaction.pan_on_scroll = true;
        s.interaction.pan_on_scroll_speed = 1.0;
        s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(100.0), Px(100.0));
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Wheel {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            delta: Point::new(Px(20.0), Px(0.0)),
            modifiers: Modifiers::default(),
            pointer_type: PointerType::Mouse,
        }),
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_start:PanScroll"));

    let token = canvas
        .interaction
        .viewport_move_debounce
        .as_ref()
        .expect("debounce timer")
        .timer;

    canvas.event(&mut cx, &Event::Timer { token });

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_end:PanScroll:Ended"));
    let _ = snapshot;
}

#[test]
fn double_click_background_zoom_emits_move_start_and_move_end() {
    let (mut host, graph, view) = make_host_graph_view(Graph::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let bounds = make_bounds();
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(10.0), Px(10.0)),
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_type: PointerType::Mouse,
        }),
    );

    let got = log.borrow().clone();
    assert!(got.iter().any(|s| s == "move_start:ZoomDoubleClick"));
    assert!(got.iter().any(|s| s == "move_end:ZoomDoubleClick:Ended"));
}

#[test]
fn wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom() {
    let (mut host, graph, view) = make_host_graph_view(Graph::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_scroll = false;
        s.interaction.pan_on_scroll = true;
        s.interaction.pan_on_scroll_speed = 1.0;
        s.interaction.pan_on_scroll_mode = crate::io::NodeGraphPanOnScrollMode::Free;
    });

    let log: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let recorder = Recorder { log: log.clone() };

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_callbacks(recorder);
    let bounds = make_bounds();
    let mut services = NullServices::default();

    let pos = Point::new(Px(100.0), Px(100.0));
    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                delta: Point::new(Px(20.0), Px(0.0)),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_scroll = true;
        s.interaction.zoom_on_scroll_speed = 1.0;
        s.interaction.zoom_activation_key = crate::io::NodeGraphZoomActivationKey::None;
        s.interaction.pan_on_scroll = false;
    });

    {
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        canvas.event(
            &mut cx,
            &Event::Pointer(PointerEvent::Wheel {
                pointer_id: fret_core::PointerId::default(),
                position: pos,
                delta: Point::new(Px(0.0), Px(-120.0)),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );
    }

    let got = log.borrow().clone();
    let pan_end = got
        .iter()
        .position(|s| s == "move_end:PanScroll:Ended")
        .expect("pan scroll ended");
    let zoom_start = got
        .iter()
        .position(|s| s == "move_start:ZoomWheel")
        .expect("zoom wheel started");
    assert!(pan_end < zoom_start);
}
