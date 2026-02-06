use fret_core::{Modifiers, MouseButtons, Point, Px, Rect, Size};

use crate::core::{
    CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind, EdgeReconnectable, EdgeReconnectableEndpoint,
    Graph, GraphId, Node, NodeId, NodeKindKey, Port, PortCapacity, PortDirection, PortId, PortKey,
    PortKind,
};
use crate::io::NodeGraphViewState;
use crate::rules::EdgeEndpoint;
use crate::ui::edge_types::{EdgeTypeKey, NodeGraphEdgeTypes};

use super::super::{
    NodeGraphCanvas, edge_drag, group_resize, left_click, marquee, node_drag, node_resize,
    pending_drag, pointer_up,
};
use super::{NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_size};
use crate::ui::canvas::state::{EdgeDrag, WireDragKind};
use crate::ui::canvas::state::{GroupResize, NodeDrag, NodeResize, NodeResizeHandle};
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
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    (graph, edge, out, inn)
}

#[test]
fn child_node_drag_is_clamped_to_group_when_expand_parent_is_false() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: crate::core::CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );

    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 10.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: None,
            expand_parent: Some(false),
            size: Some(CanvasSize {
                width: 80.0,
                height: 40.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: node_id,
        node_ids: vec![node_id],
        nodes: vec![(node_id, CanvasPoint { x: 10.0, y: 10.0 })],
        current_nodes: vec![(node_id, CanvasPoint { x: 10.0, y: 10.0 })],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(10.0), Px(10.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Try moving the node to x=80 (right edge would be 160), should clamp to max_x=20.
    assert!(node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_move = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_move.x, 10.0);

    let drag = canvas
        .interaction
        .node_drag
        .as_ref()
        .expect("node drag active");
    let preview = drag
        .current_nodes
        .iter()
        .find(|(id, _)| *id == node_id)
        .map(|(_, p)| *p)
        .unwrap();
    assert_eq!(preview.x, 20.0);

    let group_rect_after_move = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    assert_eq!(group_rect_after_move.size.width, 100.0);

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_commit = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_commit.x, 20.0);
}

#[test]
fn child_node_drag_expands_group_when_expand_parent_is_true() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: crate::core::CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );

    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 10.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: None,
            expand_parent: Some(true),
            size: Some(CanvasSize {
                width: 80.0,
                height: 40.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: node_id,
        node_ids: vec![node_id],
        nodes: vec![(node_id, CanvasPoint { x: 10.0, y: 10.0 })],
        current_nodes: vec![(node_id, CanvasPoint { x: 10.0, y: 10.0 })],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(10.0), Px(10.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Move the node to x=80 (right edge would be 160): group should expand to include it.
    assert!(node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_move = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_move.x, 10.0);

    let drag = canvas
        .interaction
        .node_drag
        .as_ref()
        .expect("node drag active");
    let preview = drag
        .current_nodes
        .iter()
        .find(|(id, _)| *id == node_id)
        .map(|(_, p)| *p)
        .unwrap();
    assert_eq!(preview.x, 80.0);

    let group_rect_preview = drag
        .current_groups
        .iter()
        .find(|(id, _)| *id == group_id)
        .map(|(_, r)| *r)
        .unwrap();
    assert_eq!(group_rect_preview.size.width, 160.0);

    let group_rect_after_move = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    assert_eq!(group_rect_after_move.size.width, 100.0);

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(10.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_commit = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_commit.x, 80.0);

    let group_rect_after_commit = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    assert_eq!(group_rect_after_commit.size.width, 160.0);
}

#[test]
fn node_drag_respects_per_node_extent_rect() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: Some(crate::core::NodeExtent::Rect {
                rect: crate::core::CanvasRect {
                    origin: CanvasPoint { x: 0.0, y: 0.0 },
                    size: CanvasSize {
                        width: 100.0,
                        height: 100.0,
                    },
                },
            }),
            expand_parent: None,
            size: Some(CanvasSize {
                width: 80.0,
                height: 40.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: node_id,
        node_ids: vec![node_id],
        nodes: vec![(node_id, CanvasPoint { x: 0.0, y: 0.0 })],
        current_nodes: vec![(node_id, CanvasPoint { x: 0.0, y: 0.0 })],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Attempt to move to x=80 (right edge would be 160); extent allows max_x=20.
    assert!(node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_move = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_move.x, 0.0);

    let drag = canvas
        .interaction
        .node_drag
        .as_ref()
        .expect("node drag active");
    let preview = drag
        .current_nodes
        .iter()
        .find(|(id, _)| *id == node_id)
        .map(|(_, p)| *p)
        .unwrap();
    assert_eq!(preview.x, 20.0);

    assert!(super::super::pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let node_pos_after_commit = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).map(|n| n.pos))
        .unwrap()
        .unwrap();
    assert_eq!(node_pos_after_commit.x, 20.0);
}

#[test]
fn multi_node_drag_clamps_by_selection_bounds_in_node_extent_rect() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let a = NodeId::new();
    let b = NodeId::new();
    graph_value.nodes.insert(
        a,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 20.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 30.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 20.0,
                height: 20.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let extent = crate::core::CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 100.0,
            height: 100.0,
        },
    };
    let mut view_state = NodeGraphViewState::default();
    view_state.interaction.node_extent = Some(extent);
    let view = host.models.insert(view_state);

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_drag = Some(NodeDrag {
        primary: a,
        node_ids: vec![a, b],
        nodes: vec![
            (a, CanvasPoint { x: 0.0, y: 0.0 }),
            (b, CanvasPoint { x: 30.0, y: 0.0 }),
        ],
        current_nodes: vec![
            (a, CanvasPoint { x: 0.0, y: 0.0 }),
            (b, CanvasPoint { x: 30.0, y: 0.0 }),
        ],
        current_groups: Vec::new(),
        preview_rev: 0,
        grab_offset: Point::new(Px(0.0), Px(0.0)),
        start_pos: Point::new(Px(0.0), Px(0.0)),
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(node_drag::handle_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let (pos_a_after_move, pos_b_after_move) = graph
        .read_ref(cx.app, |g| {
            (
                g.nodes.get(&a).map(|n| n.pos),
                g.nodes.get(&b).map(|n| n.pos),
            )
        })
        .unwrap();
    assert_eq!(pos_a_after_move.unwrap().x, 0.0);
    assert_eq!(pos_b_after_move.unwrap().x, 30.0);

    let drag = canvas
        .interaction
        .node_drag
        .as_ref()
        .expect("node drag active");
    let preview_a = drag
        .current_nodes
        .iter()
        .find(|(id, _)| *id == a)
        .map(|(_, p)| *p)
        .unwrap();
    let preview_b = drag
        .current_nodes
        .iter()
        .find(|(id, _)| *id == b)
        .map(|(_, p)| *p)
        .unwrap();

    // The selection bounds are 50px wide. Attempting dx=80 clamps to dx=50, preserving spacing.
    assert_eq!(preview_a.x, 50.0);
    assert_eq!(preview_b.x, 80.0);
    assert_eq!(preview_b.x - preview_a.x, 30.0);

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let (pos_a_after_commit, pos_b_after_commit) = graph
        .read_ref(cx.app, |g| {
            (
                g.nodes.get(&a).map(|n| n.pos),
                g.nodes.get(&b).map(|n| n.pos),
            )
        })
        .unwrap();
    assert_eq!(pos_a_after_commit.unwrap().x, 50.0);
    assert_eq!(pos_b_after_commit.unwrap().x, 80.0);
}

#[test]
fn node_resize_expands_group_when_expand_parent_is_true() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: crate::core::CanvasRect {
                origin: CanvasPoint { x: 0.0, y: 0.0 },
                size: CanvasSize {
                    width: 100.0,
                    height: 100.0,
                },
            },
            color: None,
        },
    );

    let node_id = NodeId::new();
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 10.0, y: 10.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: None,
            expand_parent: Some(true),
            size: Some(CanvasSize {
                width: 80.0,
                height: 40.0,
            }),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.node_resize = Some(NodeResize {
        node: node_id,
        handle: NodeResizeHandle::Right,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_node_pos: CanvasPoint { x: 10.0, y: 10.0 },
        start_size: CanvasSize {
            width: 80.0,
            height: 40.0,
        },
        start_size_opt: Some(CanvasSize {
            width: 80.0,
            height: 40.0,
        }),
        current_node_pos: CanvasPoint { x: 10.0, y: 10.0 },
        current_size_opt: Some(CanvasSize {
            width: 80.0,
            height: 40.0,
        }),
        current_groups: Vec::new(),
        preview_rev: 0,
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Attempt to resize width by +80 (right edge would be at x=170): group should expand.
    assert!(node_resize::handle_node_resize_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let (group_rect_after_move, node_after_move) = graph
        .read_ref(cx.app, |g| {
            (
                g.groups.get(&group_id).map(|gr| gr.rect),
                g.nodes.get(&node_id).cloned(),
            )
        })
        .unwrap();

    let group_rect_after_move = group_rect_after_move.unwrap();
    let node_after_move = node_after_move.unwrap();
    assert_eq!(node_after_move.pos, CanvasPoint { x: 10.0, y: 10.0 });
    assert_eq!(
        node_after_move.size,
        Some(CanvasSize {
            width: 80.0,
            height: 40.0,
        })
    );
    assert_eq!(
        group_rect_after_move.size.width, 100.0,
        "group should not mutate during resize preview"
    );

    let resize = canvas
        .interaction
        .node_resize
        .as_ref()
        .expect("node resize active");
    assert_eq!(resize.current_node_pos, CanvasPoint { x: 10.0, y: 10.0 });
    let size_px = resize.current_size_opt.expect("preview size");
    assert!(
        size_px.width > 80.0,
        "node width should increase in preview"
    );
    assert!(
        resize
            .current_groups
            .iter()
            .any(|(id, rect)| *id == group_id && rect.size.width > 100.0),
        "group should expand in preview"
    );

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(80.0), Px(0.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let group_rect = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    let node = graph
        .read_ref(cx.app, |g| g.nodes.get(&node_id).cloned())
        .unwrap()
        .unwrap();

    let z = snapshot.zoom.max(1.0e-6);
    let size_px = node.size.unwrap();
    let node_w_canvas = size_px.width / z;
    let right = node.pos.x + node_w_canvas;
    let group_right = group_rect.origin.x + group_rect.size.width;
    assert!(
        group_rect.size.width > 100.0,
        "group should expand after commit"
    );
    assert!(
        group_right + 1.0e-3 >= right,
        "group must contain resized node (group_right={group_right}, node_right={right})"
    );
}

#[test]
fn group_resize_is_previewed_and_committed_on_pointer_up() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    let start_rect = crate::core::CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 100.0,
            height: 100.0,
        },
    };
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: start_rect,
            color: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.group_resize = Some(GroupResize {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect,
        current_rect: start_rect,
        preview_rev: 0,
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(group_resize::handle_group_resize_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(50.0), Px(40.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let rect_after_move = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    assert_eq!(rect_after_move, start_rect);

    let resize = canvas
        .interaction
        .group_resize
        .as_ref()
        .expect("group resize active");
    assert!(resize.current_rect.size.width > start_rect.size.width);
    assert!(resize.current_rect.size.height > start_rect.size.height);

    assert!(pointer_up::handle_pointer_up(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(50.0), Px(40.0)),
        fret_core::MouseButton::Left,
        1,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let rect_after_commit = graph
        .read_ref(cx.app, |g| g.groups.get(&group_id).map(|gr| gr.rect))
        .unwrap()
        .unwrap();
    assert!(rect_after_commit.size.width > start_rect.size.width);
    assert!(rect_after_commit.size.height > start_rect.size.height);
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
fn marquee_selects_connected_edges_for_selected_nodes() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::Connected;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    // Cover only node A (at x=0..220) and exclude node B (at x=320..540).
    let end = Point::new(Px(250.0), Px(120.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
}

#[test]
fn marquee_selects_connected_edges_for_selected_nodes_with_store() {
    use crate::runtime::store::NodeGraphStore;

    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    let a = graph_value
        .ports
        .get(&from)
        .map(|p| p.node)
        .expect("from port exists");

    let mut store_view = NodeGraphViewState::default();
    store_view.interaction.elements_selectable = true;
    store_view.interaction.edges_selectable = true;
    store_view.interaction.selection_on_drag = true;
    store_view.interaction.pane_click_distance = 0.0;
    store_view.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::Connected;

    let store = host
        .models
        .insert(NodeGraphStore::new(graph_value, store_view));
    let graph = host.models.insert(Graph::default());
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_store(store.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let end = Point::new(Px(250.0), Px(120.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let selected_nodes = store
        .read_ref(&host, |s| s.view_state().selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);

    let selected_edges = store
        .read_ref(&host, |s| s.view_state().selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
}

#[test]
fn marquee_does_not_select_edges_when_edge_selectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge exists")
        .selectable = Some(false);

    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::Connected;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    // Cover only node A (at x=0..220) and exclude node B (at x=320..540).
    let end = Point::new(Px(250.0), Px(120.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert!(selected_edges.is_empty());
}

#[test]
fn marquee_does_not_select_edges_when_box_select_edges_is_none() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::None;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    let end = Point::new(Px(250.0), Px(120.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert!(selected_edges.is_empty());

    let selected_groups = view
        .read_ref(&host, |s| s.selected_groups.clone())
        .unwrap_or_default();
    assert!(selected_groups.is_empty());

    let _ = edge;
}

#[test]
fn marquee_selects_edges_only_when_both_endpoints_selected_in_both_endpoints_mode() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let b = graph
        .read_ref(&host, |g| g.ports.get(&to).map(|p| p.node))
        .ok()
        .flatten()
        .expect("to port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::BothEndpoints;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();

    // Cover only node A (exclude B) -> edge should not be selected.
    {
        let snapshot = canvas.sync_view_state(&mut host);
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        let start = Point::new(Px(-10.0), Px(-10.0));
        assert!(left_click::handle_left_click_pointer_down(
            &mut canvas,
            &mut cx,
            &snapshot,
            start,
            Modifiers::default(),
            snapshot.zoom,
        ));
        let end = Point::new(Px(250.0), Px(120.0));
        assert!(marquee::handle_marquee_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            end,
            Modifiers::default(),
            snapshot.zoom,
        ));
        assert!(marquee::handle_left_up(&mut canvas, &mut cx));
    }

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);
    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert!(selected_edges.is_empty());

    // Cover both A and B -> edge should be selected.
    {
        let snapshot = canvas.sync_view_state(&mut host);
        let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
        let mut cx = event_cx(
            &mut host,
            &mut services,
            bounds,
            &mut prevented_default_actions,
        );
        let start = Point::new(Px(-10.0), Px(-10.0));
        assert!(left_click::handle_left_click_pointer_down(
            &mut canvas,
            &mut cx,
            &snapshot,
            start,
            Modifiers::default(),
            snapshot.zoom,
        ));
        let end = Point::new(Px(600.0), Px(120.0));
        assert!(marquee::handle_marquee_move(
            &mut canvas,
            &mut cx,
            &snapshot,
            end,
            Modifiers::default(),
            snapshot.zoom,
        ));
        assert!(marquee::handle_left_up(&mut canvas, &mut cx));
    }

    let mut selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    selected_nodes.sort();
    let mut expected = vec![a, b];
    expected.sort();
    assert_eq!(selected_nodes, expected);

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
fn edge_click_clears_node_selection_when_not_in_multi_select_mode() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.selected_nodes = vec![a];
        s.selected_edges.clear();
        s.selected_groups.clear();
    });

    let edge_types =
        NodeGraphEdgeTypes::new().register(EdgeTypeKey::new("data"), |_g, _e, _s, mut h| {
            h.route = crate::ui::presenter::EdgeRouteKind::Straight;
            h
        });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_edge_types(edge_types);
    let snapshot = canvas.sync_view_state(&mut host);

    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let pos = Point::new(
        Px(0.5 * (from_center.x.0 + to_center.x.0)),
        Px(0.5 * (from_center.y.0 + to_center.y.0)),
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

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

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert!(
        selected_nodes.is_empty(),
        "clicking an edge should clear node selection in non-multi mode"
    );

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
}

#[test]
fn edge_click_does_not_select_edge_when_edge_selectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge exists")
        .selectable = Some(false);

    let graph = host.models.insert(graph_value);
    let a = graph
        .read_ref(&host, |g| g.ports.get(&from).map(|p| p.node))
        .ok()
        .flatten()
        .expect("from port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.selected_nodes = vec![a];
        s.selected_edges.clear();
        s.selected_groups.clear();
    });

    let edge_types =
        NodeGraphEdgeTypes::new().register(EdgeTypeKey::new("data"), |_g, _e, _s, mut h| {
            h.route = crate::ui::presenter::EdgeRouteKind::Straight;
            h
        });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_edge_types(edge_types);
    let snapshot = canvas.sync_view_state(&mut host);

    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let pos = Point::new(
        Px(0.5 * (from_center.x.0 + to_center.x.0)),
        Px(0.5 * (from_center.y.0 + to_center.y.0)),
    );

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

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

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![a]);

    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert!(selected_edges.is_empty());
}

#[test]
fn node_click_does_not_select_node_when_node_selectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _edge, from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert({
        let a = graph_value
            .ports
            .get(&from)
            .map(|p| p.node)
            .expect("from port exists");
        graph_value
            .nodes
            .get_mut(&a)
            .expect("node exists")
            .selectable = Some(false);
        graph_value
    });
    let b = graph
        .read_ref(&host, |g| g.ports.get(&to).map(|p| p.node))
        .ok()
        .flatten()
        .expect("to port exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.selected_nodes = vec![b];
        s.selected_edges.clear();
        s.selected_groups.clear();
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    // Node A is at (0, 0) with size (220, 80).
    let pos = Point::new(Px(110.0), Px(40.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

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

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![b]);
}

#[test]
fn node_drag_does_not_start_when_node_draggable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node exists")
        .draggable = Some(false);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.nodes_draggable = true;
        s.interaction.node_drag_threshold = 0.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(5.0), Px(2.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(canvas.interaction.pending_node_drag.is_some());

    let moved = Point::new(Px(120.0), Px(60.0));
    assert!(pending_drag::handle_pending_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        moved,
        snapshot.zoom,
    ));

    assert!(canvas.interaction.node_drag.is_none());
    assert!(canvas.interaction.pending_node_drag.is_none());

    let a_pos = graph
        .read_ref(&host, |g| g.nodes.get(&a).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap();
    assert_eq!(a_pos, CanvasPoint { x: 0.0, y: 0.0 });
}

#[test]
fn node_drag_does_not_start_when_nodes_draggable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _b) = make_test_graph_two_nodes_with_size();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.nodes_draggable = false;
        s.interaction.node_drag_threshold = 0.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(5.0), Px(2.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(canvas.interaction.pending_node_drag.is_some());

    let moved = Point::new(Px(120.0), Px(60.0));
    assert!(pending_drag::handle_pending_node_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        moved,
        snapshot.zoom,
    ));

    assert!(canvas.interaction.node_drag.is_none());
    assert!(canvas.interaction.pending_node_drag.is_none());

    let a_pos = graph
        .read_ref(&host, |g| g.nodes.get(&a).map(|n| n.pos))
        .ok()
        .flatten()
        .unwrap();
    assert_eq!(a_pos, CanvasPoint { x: 0.0, y: 0.0 });
}

#[test]
fn marquee_does_not_select_nodes_when_node_selectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    let a = graph_value
        .ports
        .get(&from)
        .map(|p| p.node)
        .expect("from port exists");
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node exists")
        .selectable = Some(false);

    let graph = host.models.insert(graph_value);
    let b = graph
        .read_ref(&host, |g| {
            g.edges
                .get(&edge)
                .and_then(|e| g.ports.get(&e.to))
                .map(|p| p.node)
        })
        .ok()
        .flatten()
        .expect("to node exists");
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.elements_selectable = true;
        s.interaction.edges_selectable = true;
        s.interaction.selection_on_drag = true;
        s.interaction.pane_click_distance = 0.0;
        s.interaction.box_select_edges = crate::io::NodeGraphBoxSelectEdges::Connected;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let start = Point::new(Px(-10.0), Px(-10.0));
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        start,
        Modifiers::default(),
        snapshot.zoom,
    ));

    // Cover both nodes A and B.
    let end = Point::new(Px(600.0), Px(120.0));
    assert!(marquee::handle_marquee_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        end,
        Modifiers::default(),
        snapshot.zoom,
    ));
    assert!(marquee::handle_left_up(&mut canvas, &mut cx));

    let selected_nodes = view
        .read_ref(&host, |s| s.selected_nodes.clone())
        .unwrap_or_default();
    assert_eq!(selected_nodes, vec![b]);

    // Edge is still connected to B, so should be selected.
    let selected_edges = view
        .read_ref(&host, |s| s.selected_edges.clone())
        .unwrap_or_default();
    assert_eq!(selected_edges, vec![edge]);
}

#[test]
fn port_click_does_not_start_wire_drag_when_nodes_connectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _edge, from, _to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_connectable = false;
        s.interaction.connect_on_click = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let pos = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_wire_drag.is_none());
    assert!(canvas.interaction.wire_drag.is_none());
}

#[test]
fn port_click_starts_wire_drag_when_node_connectable_true_even_if_nodes_connectable_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _edge, from, _to) = make_test_graph_edge_reconnect();
    let from_node = graph_value
        .ports
        .get(&from)
        .map(|p| p.node)
        .expect("from port must exist");
    graph_value
        .nodes
        .get_mut(&from_node)
        .expect("from node must exist")
        .connectable = Some(true);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_connectable = false;
        s.interaction.connect_on_click = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let pos = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_wire_drag.is_some());
    assert!(canvas.interaction.wire_drag.is_none());
}

#[test]
fn port_click_does_not_start_wire_drag_when_port_connectable_start_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _edge, from, _to) = make_test_graph_edge_reconnect();
    graph_value
        .ports
        .get_mut(&from)
        .expect("from port must exist")
        .connectable_start = Some(false);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_connectable = true;
        s.interaction.connect_on_click = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let pos = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_wire_drag.is_none());
    assert!(canvas.interaction.wire_drag.is_none());
}

#[test]
fn ctrl_click_port_yanks_edges_and_starts_reconnect_with_store() {
    use crate::runtime::store::NodeGraphStore;

    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, from, to) = make_test_graph_edge_reconnect();

    let mut store_view = NodeGraphViewState::default();
    store_view.interaction.nodes_connectable = true;
    store_view.interaction.edges_reconnectable = true;
    store_view.interaction.connect_on_click = false;

    let store = host
        .models
        .insert(NodeGraphStore::new(graph_value.clone(), store_view));
    let graph = host.models.insert(Graph::default());
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_store(store);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let pos = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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

    let Some(pending) = canvas.interaction.pending_wire_drag.as_ref() else {
        panic!("expected pending wire drag");
    };
    match &pending.kind {
        WireDragKind::Reconnect {
            edge: got_edge,
            endpoint,
            fixed,
        } => {
            assert_eq!(*got_edge, edge);
            assert_eq!(*endpoint, EdgeEndpoint::From);
            assert_eq!(*fixed, to);
        }
        other => panic!("unexpected kind: {other:?}"),
    }
}

#[test]
fn port_connectable_override_allows_start_even_when_nodes_connectable_is_false() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _edge, from, _to) = make_test_graph_edge_reconnect();
    graph_value
        .ports
        .get_mut(&from)
        .expect("from port must exist")
        .connectable = Some(true);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_connectable = false;
        s.interaction.connect_on_click = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let pos = geom.port_center(from).expect("from port center");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    assert!(left_click::handle_left_click_pointer_down(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos,
        Modifiers::default(),
        snapshot.zoom,
    ));

    assert!(canvas.interaction.pending_wire_drag.is_some());
    assert!(canvas.interaction.wire_drag.is_none());
}

#[test]
fn pick_target_port_respects_port_connectable_end() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _edge, from, to) = make_test_graph_edge_reconnect();
    graph_value
        .ports
        .get_mut(&to)
        .expect("to port must exist")
        .connectable_end = Some(false);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.nodes_connectable = true;
        s.interaction.connection_mode = crate::interaction::NodeGraphConnectionMode::Strict;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let to_center = geom.port_center(to).expect("to port center");

    let (derived, index) = canvas.canvas_derived(&host, &snapshot);
    let hit = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = super::super::HitTestScratch::default();
            let mut ctx = super::super::HitTestCtx::new(
                derived.as_ref(),
                index.as_ref(),
                snapshot.zoom,
                &mut scratch,
            );
            canvas.pick_target_port(g, &snapshot, &mut ctx, from, true, to_center)
        })
        .ok()
        .flatten();
    assert!(hit.is_none());
}

#[test]
fn connectable_false_prevents_connecting_to_target_port() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let out = PortId::new();
    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
            collapsed: false,
            ports: vec![out],
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let b = NodeId::new();
    let inn = PortId::new();
    graph_value.nodes.insert(
        b,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 320.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: Some(false),
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: Some(CanvasSize {
                width: 220.0,
                height: 80.0,
            }),
            hidden: false,
            collapsed: false,
            ports: vec![inn],
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        inn,
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

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    let snapshot = canvas.sync_view_state(&mut host);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
        kind: WireDragKind::New {
            from: out,
            bundle: vec![out],
        },
        pos: Point::new(Px(0.0), Px(0.0)),
    });

    assert!(
        super::super::wire_drag::handle_wire_left_up_with_forced_target(
            &mut canvas,
            &mut cx,
            &snapshot,
            snapshot.zoom,
            Some(inn),
        )
    );

    assert_eq!(graph.read_ref(&host, |g| g.edges.len()).unwrap_or(0), 0);
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
fn edge_reconnect_drag_cancels_when_endpoint_not_reconnectable() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, edge, from, _to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge must exist")
        .reconnectable = Some(EdgeReconnectable::Endpoint(
        EdgeReconnectableEndpoint::Target,
    ));

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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.interaction.edge_drag = Some(EdgeDrag {
        edge,
        start_pos: from_center,
    });

    let t = snapshot.interaction.connection_drag_threshold.max(1.0);
    let pos_big = Point::new(Px(from_center.x.0 + t + 1.0), from_center.y);
    assert!(edge_drag::handle_edge_drag_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        pos_big,
        snapshot.zoom,
    ));

    assert!(canvas.interaction.wire_drag.is_none());
    assert!(canvas.interaction.edge_drag.is_none());
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn edge_reconnectable_endpoint_override_allows_anchors_even_when_global_is_disabled() {
    let mut host = TestUiHostImpl::default();

    let (mut graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge must exist")
        .reconnectable = Some(EdgeReconnectable::Endpoint(
        EdgeReconnectableEndpoint::Source,
    ));

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let route = canvas
        .graph
        .read_ref(&host, |g| canvas.edge_render_hint(g, edge).route)
        .unwrap_or_default();
    let (a0, a1) =
        NodeGraphCanvas::edge_focus_anchor_centers(route, from_center, to_center, snapshot.zoom);

    let (derived, index) = canvas.canvas_derived(&host, &snapshot);
    let hit_source = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = super::super::HitTestScratch::default();
            let mut ctx = super::super::HitTestCtx::new(
                derived.as_ref(),
                index.as_ref(),
                snapshot.zoom,
                &mut scratch,
            );
            canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, a0)
        })
        .ok()
        .flatten();
    assert!(hit_source.is_some_and(|(id, ep, _)| id == edge && ep == EdgeEndpoint::From));

    let hit_target = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = super::super::HitTestScratch::default();
            let mut ctx = super::super::HitTestCtx::new(
                derived.as_ref(),
                index.as_ref(),
                snapshot.zoom,
                &mut scratch,
            );
            canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, a1)
        })
        .ok()
        .flatten();
    assert!(hit_target.is_none(), "target anchor should be disabled");
}

#[test]
fn edge_reconnectable_target_override_allows_only_target_anchor_when_global_disabled() {
    let mut host = TestUiHostImpl::default();

    let (mut graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge must exist")
        .reconnectable = Some(EdgeReconnectable::Endpoint(
        EdgeReconnectableEndpoint::Target,
    ));

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let route = canvas
        .graph
        .read_ref(&host, |g| canvas.edge_render_hint(g, edge).route)
        .unwrap_or_default();
    let (a0, a1) =
        NodeGraphCanvas::edge_focus_anchor_centers(route, from_center, to_center, snapshot.zoom);

    let (derived, index) = canvas.canvas_derived(&host, &snapshot);
    let hit_source = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = super::super::HitTestScratch::default();
            let mut ctx = super::super::HitTestCtx::new(
                derived.as_ref(),
                index.as_ref(),
                snapshot.zoom,
                &mut scratch,
            );
            canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, a0)
        })
        .ok()
        .flatten();
    assert!(hit_source.is_none(), "source anchor should be disabled");

    let hit_target = canvas
        .graph
        .read_ref(&host, |g| {
            let mut scratch = super::super::HitTestScratch::default();
            let mut ctx = super::super::HitTestCtx::new(
                derived.as_ref(),
                index.as_ref(),
                snapshot.zoom,
                &mut scratch,
            );
            canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, a1)
        })
        .ok()
        .flatten();
    assert!(hit_target.is_some_and(|(id, ep, _)| id == edge && ep == EdgeEndpoint::To));
}

#[test]
fn edge_reconnectable_bool_false_disables_anchors_even_when_global_enabled() {
    let mut host = TestUiHostImpl::default();

    let (mut graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    graph_value
        .edges
        .get_mut(&edge)
        .expect("edge must exist")
        .reconnectable = Some(EdgeReconnectable::Bool(false));

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = true;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let route = canvas
        .graph
        .read_ref(&host, |g| canvas.edge_render_hint(g, edge).route)
        .unwrap_or_default();
    let (a0, a1) =
        NodeGraphCanvas::edge_focus_anchor_centers(route, from_center, to_center, snapshot.zoom);

    let (derived, index) = canvas.canvas_derived(&host, &snapshot);
    for anchor in [a0, a1] {
        let hit = canvas
            .graph
            .read_ref(&host, |g| {
                let mut scratch = super::super::HitTestScratch::default();
                let mut ctx = super::super::HitTestCtx::new(
                    derived.as_ref(),
                    index.as_ref(),
                    snapshot.zoom,
                    &mut scratch,
                );
                canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, anchor)
            })
            .ok()
            .flatten();
        assert!(hit.is_none(), "anchors should be disabled");
    }
}

#[test]
fn edge_reconnectable_none_follows_global_gate_for_anchors() {
    let mut host = TestUiHostImpl::default();

    let (graph_value, edge, from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edges_reconnectable = false;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);

    let from_center = geom.port_center(from).expect("from port center");
    let to_center = geom.port_center(to).expect("to port center");
    let route = canvas
        .graph
        .read_ref(&host, |g| canvas.edge_render_hint(g, edge).route)
        .unwrap_or_default();
    let (a0, a1) =
        NodeGraphCanvas::edge_focus_anchor_centers(route, from_center, to_center, snapshot.zoom);

    let (derived, index) = canvas.canvas_derived(&host, &snapshot);
    for anchor in [a0, a1] {
        let hit = canvas
            .graph
            .read_ref(&host, |g| {
                let mut scratch = super::super::HitTestScratch::default();
                let mut ctx = super::super::HitTestCtx::new(
                    derived.as_ref(),
                    index.as_ref(),
                    snapshot.zoom,
                    &mut scratch,
                );
                canvas.hit_edge_focus_anchor(g, &snapshot, &mut ctx, anchor)
            })
            .ok()
            .flatten();
        assert!(hit.is_none(), "anchors should be disabled");
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
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
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
            pointer_id: fret_core::PointerId::default(),
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.interaction.node_drag = Some(crate::ui::canvas::state::NodeDrag {
        primary: a,
        node_ids: vec![a],
        nodes: vec![(a, CanvasPoint { x: 0.0, y: 0.0 })],
        current_nodes: vec![(a, CanvasPoint { x: 0.0, y: 0.0 })],
        current_groups: Vec::new(),
        preview_rev: 0,
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
            pointer_id: fret_core::PointerId::default(),
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
fn missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_wire_reconnect_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
        kind: WireDragKind::Reconnect {
            edge,
            endpoint: EdgeEndpoint::From,
            fixed: to,
        },
        pos: Point::new(Px(40.0), Px(10.0)),
    });
    assert!(canvas.interaction.wire_drag.is_some());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    // Simulate a missed `PointerEvent::Up`: Move arrives with no left button held.
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(40.0), Px(10.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        canvas.interaction.wire_drag.is_none(),
        "expected the inferred-up path to end the active reconnect drag"
    );
    let edge_still_exists = canvas
        .graph
        .read_ref(cx.app, |g| g.edges.contains_key(&edge))
        .unwrap_or(false);
    assert!(edge_still_exists);
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_new_wire_drag() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _edge, from, _to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);
    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
        kind: WireDragKind::New {
            from,
            bundle: Vec::new(),
        },
        pos: Point::new(Px(700.0), Px(500.0)),
    });
    assert!(canvas.interaction.wire_drag.is_some());

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );

    // Simulate a missed `PointerEvent::Up`: Move arrives with no left button held.
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Move {
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(700.0), Px(500.0)),
            buttons: MouseButtons::default(),
            modifiers: Modifiers::default(),
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(
        canvas.interaction.wire_drag.is_none(),
        "expected the inferred-up path to end the active connect drag"
    );
    assert!(
        canvas.interaction.suspended_wire_drag.is_some(),
        "expected a connect drag drop on empty to suspend and open the insert-node picker"
    );
    assert!(
        canvas.interaction.searcher.is_some(),
        "expected inferred-up behavior to flow through the canonical wire-drag pointer-up path"
    );
    assert_eq!(canvas.history.undo_len(), 0);
}

#[test]
fn right_click_cancels_wire_drag_and_opens_context_menu() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, edge, _from, to) = make_test_graph_edge_reconnect();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view);

    canvas.interaction.wire_drag = Some(crate::ui::canvas::state::WireDrag {
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let pos = Point::new(Px(400.0), Px(300.0));
    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
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
            pointer_id: fret_core::PointerId::default(),
            position: pos,
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: true,
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
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let mut cx = event_cx(
        &mut host,
        &mut services,
        bounds,
        &mut prevented_default_actions,
    );
    let mut snapshot = canvas.sync_view_state(cx.app);
    let start_screen = Point::new(Px(100.0), Px(100.0));
    let start_local = start_screen;

    canvas.event(
        &mut cx,
        &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
            pointer_id: fret_core::PointerId::default(),
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
            pointer_id: fret_core::PointerId::default(),
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
                pointer_id: fret_core::PointerId::default(),
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
            pointer_id: fret_core::PointerId::default(),
            position: Point::new(Px(0.0), Px(0.0)),
            button: fret_core::MouseButton::Right,
            modifiers: Modifiers::default(),
            is_click: false,
            click_count: 1,
            pointer_type: fret_core::PointerType::Mouse,
        }),
    );

    assert!(canvas.interaction.context_menu.is_none());
    assert_eq!(canvas.history.undo_len(), 0);
}
