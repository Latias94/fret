use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey,
    Port, PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphViewState;
use crate::rules::EdgeEndpoint;

use super::super::super::state::{EdgeDrag, WireDragKind};
use super::super::{NodeGraphCanvas, edge_drag, left_click, marquee, pointer_up};
use super::{NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_size};

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
fn marquee_toggle_mode_toggles_nodes_in_rect() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.selected_nodes = vec![a];
        s.interaction.elements_selectable = true;
        s.interaction.node_drag_threshold = 0.0;
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
    assert_eq!(selected, vec![b]);
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
