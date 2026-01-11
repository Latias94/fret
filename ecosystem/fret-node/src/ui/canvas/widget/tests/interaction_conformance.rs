use fret_core::{Modifiers, MouseButtons, Point, Px, Rect, Size};

use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey,
    Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphViewState;
use crate::rules::EdgeEndpoint;

use super::super::super::state::{EdgeDrag, WireDragKind};
use super::super::{NodeGraphCanvas, edge_drag, left_click, marquee, pointer_up};
use super::{NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_size};
use fret_ui::retained_bridge::Widget as _;

fn make_test_graph_edge_reconnect() -> (Graph, EdgeId, PortId, PortId) {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let out = PortId::new();
    graph.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            collapsed: false,
            ports: vec![out],
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let b = NodeId::new();
    let inn = PortId::new();
    graph.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 320.0, y: 0.0 },
            parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            collapsed: false,
            ports: vec![inn],
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        inn,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let edge = EdgeId::new();
    graph.edges.insert(
        edge,
        Edge {
            kind: EdgeKind::Data,
            from: out,
            to: inn,
        },
    );

    (graph, edge, out, inn)
}

#[test]
fn background_click_does_not_start_marquee_when_elements_not_selectable() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.interaction.elements_selectable = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(600.0), Px(500.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_marquee.is_none());
    assert!(canvas.interaction.marquee.is_none());

    let _ = pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    );

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}

#[test]
fn background_click_starts_pending_marquee_and_clears_selection_on_up() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.interaction.elements_selectable = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(600.0), Px(500.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_marquee.is_some());

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert!(selected.is_empty());
}

#[test]
fn shift_clicking_a_node_does_not_clear_selection() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.interaction.elements_selectable = true;
        s.interaction.selection_on_drag = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(20.0), Px(10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_marquee.is_some());
    assert!(canvas.interaction.pending_node_drag.is_none());

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        fret_core::MouseButton::Left,
        1,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    let selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected, vec![a]);
}

#[test]
fn marquee_replace_mode_replaces_selection_even_with_ctrl_pressed() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.interaction.elements_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    let end = Point::new(Px(80.0), Px(40.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let mut selected = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    selected.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(selected, expected);
}

#[test]
fn multi_selection_active_does_not_clear_edge_selection_when_clicking_node() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let b = graph
        .read_ref(&host, |g| g.ports.get(&to).map(|p| p.node))
        .ok()
        .flatten()
        .expect("port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.selected_edges = vec![edge];
        s.selected_nodes.clear();
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    // Click inside node B with multi-selection key held.
    // (In `make_test_graph_edge_reconnect`, node B is at (320, 0) with size (220, 80).)
    let pos = Point::new(Px(330.0), Px(10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        fret_core::MouseButton::Left,
        1,
        Modifiers {
            ctrl: true,
            ..Modifiers::default()
        },
        snapshot.zoom,
    ));

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);

    let mut selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    selected_nodes.sort();
    assert_eq!(selected_nodes, vec![b]);
}

#[test]
fn edge_reconnect_requires_drag_threshold_before_starting_wire_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);

    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from_center = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.interaction.edge_drag = Some(EdgeDrag {
        edge,
        start_pos: from_center,
    });

    let t = snapshot.interaction.connection_drag_threshold.max(1.0);
    let pos_small = Point::new(Px(from_center.x.0 + t - 0.1), from_center.y);
    assert!(!edge_drag::handle_edge_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos_small,
        snapshot.zoom,
    ));
    assert!(canvas.interaction.wire_drag.is_none());
    assert!(canvas.interaction.edge_drag.is_some());
    assert_eq!(canvas.history.undo_len(), 0);

    let pos_big = Point::new(Px(from_center.x.0 + t + 1.0), from_center.y);
    assert!(edge_drag::handle_edge_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos_big,
        snapshot.zoom,
    ));
    assert!(canvas.interaction.edge_drag.is_none());
    assert_eq!(canvas.history.undo_len(), 0);

    let Some(w) = canvas.interaction.wire_drag.as_ref() else {
        panic!("expected wire_drag to start");
    };
    match &w.kind {
        WireDragKind::Reconnect {
            edge: e,
            endpoint,
            fixed,
        } => {
            assert_eq!(*e, edge);
            assert_eq!(*endpoint, EdgeEndpoint::From);
            assert_eq!(*fixed, to);
        }
        other => panic!("unexpected wire drag kind: {other:?}"),
    }
}

#[test]
fn edge_reconnect_drop_on_empty_can_disconnect_edge() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.reconnect_on_drop_empty = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);
    canvas.interaction.wire_drag = Some(super::super::super::state::WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::From,
            fixed: to,
        },
        pos: Point::new(Px(10_000.0), Px(10_000.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    assert!(super::super::wire_drag::handle_wire_left_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        snapshot.zoom,
    ));

    let edges_len = graph.read_ref(&host, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(edges_len, 0);
    assert_eq!(canvas.history.undo_len(), 1);
}

#[test]
fn window_focus_lost_cancels_wire_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    canvas.interaction.wire_drag = Some(super::super::super::state::WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::From,
            fixed: to,
        },
        pos: Point::new(Px(10.0), Px(10.0)),
    });
    assert!(canvas.interaction.wire_drag.is_some());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.event(&mut cx, &fret_core::Event::WindowFocusChanged(false));
    assert!(canvas.interaction.wire_drag.is_none());

    // Graph should remain unchanged (disconnect on drop empty is a separate opt-in behavior).
    let edge_still_exists = canvas
        .graph
        .read_ref(cx.app, |g| g.edges.contains_key(&edge))
        .unwrap_or(false);
    assert!(edge_still_exists);

    // Cancel should not change graph history.
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn pointer_left_cancels_wire_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    canvas.interaction.wire_drag = Some(super::super::super::state::WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::From,
            fixed: to,
        },
        pos: Point::new(Px(10.0), Px(10.0)),
    });
    assert!(canvas.interaction.wire_drag.is_some());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.event(
        &mut cx,
        &fret_core::Event::PointerCancel(fret_core::PointerCancelEvent {
            position: None,
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
            reason: fret_core::PointerCancelReason::LeftWindow,
        }),
    );
    assert!(canvas.interaction.wire_drag.is_none());

    // Graph should remain unchanged (disconnect on drop empty is a separate opt-in behavior).
    let edge_still_exists = canvas
        .graph
        .read_ref(cx.app, |g| g.edges.contains_key(&edge))
        .unwrap_or(false);
    assert!(edge_still_exists);

    // Cancel should not change graph history.
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn missing_pointer_up_can_be_inferred_from_mouse_buttons_state() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.snaplines = false;
        s.interaction.snap_to_grid = false;
        s.interaction.auto_pan.on_node_drag = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.interaction.node_drag = Some(super::super::super::state::NodeDrag {
        primary: a,
        nodes: vec![(a, CanvasPoint { x: 0.0, y: 0.0 })],
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    assert!(super::super::node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(40.0), Px(10.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert_eq!(canvas.history.undo_len(), 0);

    // Simulate a missed `PointerEvent::Up`: Move arrives with no left button held.
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: Point::new(Px(40.0), Px(10.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.node_drag.is_none());
    assert_eq!(canvas.history.undo_len(), 1);

    let pos = graph
        .read_ref(cx.app, |g| g.nodes.get(&a).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(pos, CanvasPoint { x: 40.0, y: 10.0 });
}

#[test]
fn right_click_cancels_wire_drag_and_opens_context_menu() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);

    canvas.interaction.wire_drag = Some(super::super::super::state::WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::From,
            fixed: to,
        },
        pos: Point::new(Px(10.0), Px(10.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: Point::new(Px(400.0), Px(300.0)),
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.wire_drag.is_none());
    assert!(canvas.interaction.context_menu.is_some());
}

#[test]
fn right_pan_defers_context_menu_until_pointer_up() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_on_drag.right = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let pos = Point::new(Px(400.0), Px(300.0));
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: pos,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.context_menu.is_none());
    assert!(canvas.interaction.pending_right_click.is_some());

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: pos,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.context_menu.is_some());
}

#[test]
fn right_pan_drag_does_not_open_context_menu() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.pan_on_drag.right = true;
        s.interaction.pan_on_scroll = false;
        s.interaction.pane_click_distance = 2.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut cx = event_cx(&mut host, &mut services, bounds);

    let mut snapshot = canvas.sync_view_state(cx.app);
    let start_screen = Point::new(Px(100.0), Px(100.0));
    let start_local = start_screen;

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            position: start_local,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    // Exceed click distance so we start panning on the next move.
    let first_screen = Point::new(Px(110.0), Px(100.0));
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            position: first_screen,
            buttons: MouseButtons {
                right: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );
    assert!(canvas.interaction.panning);

    // Now pan should apply on subsequent moves; provide local positions matching stable screen
    // motion under render_transform.
    let screen_positions = [
        Point::new(Px(110.0), Px(100.0)),
        Point::new(Px(160.0), Px(100.0)),
    ];
    for screen in screen_positions {
        let zoom = snapshot.zoom;
        let pan = snapshot.pan;
        let local = Point::new(
            Px((screen.x.0 - bounds.origin.x.0) / zoom - pan.x),
            Px((screen.y.0 - bounds.origin.y.0) / zoom - pan.y),
        );
        canvas.event(
            &mut cx,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
                position: local,
                buttons: MouseButtons {
                    right: true,
                    ..MouseButtons::default()
                },
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        snapshot = canvas.sync_view_state(cx.app);
    }

    assert!(snapshot.pan.x.abs() > 0.1);

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
            position: Point::new(Px(0.0), Px(0.0)),
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.context_menu.is_none());
    assert_eq!(canvas.history.undo_len(), 0);
}
