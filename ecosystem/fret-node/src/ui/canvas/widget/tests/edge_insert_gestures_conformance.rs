use fret_core::{Event, Modifiers, MouseButton, MouseButtons, Point, PointerEvent, Px, Rect, Size};
use fret_ui::retained_bridge::Widget;

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphCanvas;

use super::super::{cubic_bezier, wire_ctrl_points};
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn edge_midpoint_pos(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    from: crate::core::PortId,
    to: crate::core::PortId,
) -> Point {
    let snap = canvas.sync_view_state(host);
    let (geom, _index) = canvas.canvas_derived(host, &snap);
    let from = geom.port_center(from).expect("from port center");
    let to = geom.port_center(to).expect("to port center");
    let (c1, c2) = wire_ctrl_points(from, to, snap.zoom);
    cubic_bezier(from, c1, c2, to, 0.5)
}

#[test]
fn double_click_edge_inserts_reroute_when_enabled() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
        s.interaction.reroute_on_edge_double_click = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let pos = edge_midpoint_pos(&mut canvas, cx.app, a_out, b_in);
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers::default(),
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(nodes_len, 3);
    assert_eq!(edges_len, 2);
    assert!(
        graph
            .read_ref(cx.app, |g| g
                .nodes
                .values()
                .any(|n| n.kind.0 == crate::REROUTE_KIND))
            .unwrap_or(false)
    );

    let after = canvas.sync_view_state(cx.app);
    assert_eq!(after.selected_edges.len(), 0);
    assert_eq!(after.selected_nodes.len(), 1);
    assert_eq!(after.zoom, 1.0);
}

#[test]
fn alt_double_click_edge_opens_insert_node_picker() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let pos = edge_midpoint_pos(&mut canvas, cx.app, a_out, b_in);
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(nodes_len, 2);
    assert_eq!(edges_len, 1);

    let Some(searcher) = canvas.interaction.searcher.as_ref() else {
        panic!("expected searcher to be open");
    };
    assert!(matches!(
        searcher.target,
        crate::ui::canvas::state::ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_id
    ));
}

#[test]
fn alt_double_click_edge_prefers_picker_over_reroute_when_both_enabled() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.zoom_on_double_click = true;
        s.interaction.reroute_on_edge_double_click = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let pos = edge_midpoint_pos(&mut canvas, cx.app, a_out, b_in);
    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: MouseButton::Left,
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            click_count: 2,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(
        nodes_len, 2,
        "expected alt+double-click to not insert reroute"
    );
    assert_eq!(
        edges_len, 1,
        "expected alt+double-click to not split the edge"
    );

    let Some(searcher) = canvas.interaction.searcher.as_ref() else {
        panic!("expected searcher to be open");
    };
    assert!(matches!(
        searcher.target,
        crate::ui::canvas::state::ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_id
    ));
}

#[test]
fn alt_drag_edge_opens_insert_node_picker_when_enabled() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(420.0);
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edge_insert_on_alt_drag = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let edge_pos = edge_midpoint_pos(&mut canvas, cx.app, a_out, b_in);

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
            position: edge_pos,
            button: MouseButton::Left,
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(edge_pos.x.0 + 16.0), edge_pos.y),
            buttons: MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    canvas.event(
        &mut cx,
        &Event::Pointer(PointerEvent::Up {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(edge_pos.x.0 + 16.0), edge_pos.y),
            button: MouseButton::Left,
            modifiers: Modifiers {
                alt: true,
                ..Modifiers::default()
            },
            is_click: true,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    let nodes_len = graph.read_ref(cx.app, |g| g.nodes.len()).unwrap_or(0);
    let edges_len = graph.read_ref(cx.app, |g| g.edges.len()).unwrap_or(0);
    assert_eq!(nodes_len, 2);
    assert_eq!(edges_len, 1);

    let Some(searcher) = canvas.interaction.searcher.as_ref() else {
        panic!("expected searcher to be open");
    };
    assert!(matches!(
        searcher.target,
        crate::ui::canvas::state::ContextMenuTarget::EdgeInsertNodePicker(e) if e == edge_id
    ));
}
