use std::sync::Arc;

use fret_core::{Modifiers, Point, Px, Rect, Size};

use crate::core::{
    CanvasPoint, CanvasRect, CanvasSize, Edge, EdgeId, EdgeKind, Graph, GraphId, Node,
};
use crate::io::NodeGraphViewState;
use crate::ui::canvas::state::{GroupDrag, GroupResize};

use super::super::NodeGraphCanvas;
use super::{
    NullServices, TestUiHostImpl, event_cx, make_test_graph_two_nodes_with_ports_spaced_x,
};

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
}

#[test]
fn group_rect_with_preview_prefers_group_resize_over_group_drag() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    let base = CanvasRect {
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
            rect: base,
            color: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let drag_rect = CanvasRect {
        origin: CanvasPoint { x: 10.0, y: 20.0 },
        size: base.size,
    };
    let resize_rect = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 777.0,
            height: 333.0,
        },
    };

    canvas.interaction.group_drag = Some(GroupDrag {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: base,
        nodes: Vec::new(),
        current_rect: drag_rect,
        current_nodes: Vec::new(),
        preview_rev: 0,
    });
    canvas.interaction.group_resize = Some(GroupResize {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: base,
        current_rect: resize_rect,
        preview_rev: 0,
    });

    let got = canvas.group_rect_with_preview(group_id, base);
    assert_eq!(got, resize_rect);
}

#[test]
fn group_rect_with_preview_uses_group_drag_current_rect() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    let base = CanvasRect {
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
            rect: base,
            color: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let drag_rect = CanvasRect {
        origin: CanvasPoint {
            x: 123.0,
            y: -456.0,
        },
        size: base.size,
    };
    canvas.interaction.group_drag = Some(GroupDrag {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: base,
        nodes: Vec::new(),
        current_rect: drag_rect,
        current_nodes: Vec::new(),
        preview_rev: 0,
    });

    let got = canvas.group_rect_with_preview(group_id, base);
    assert_eq!(got, drag_rect);
}

#[test]
fn group_drag_drives_canvas_derived_preview_and_edge_index() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, _a_in, a_out, b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);

    let group_id = crate::core::GroupId::new();
    let group_rect = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 800.0,
            height: 400.0,
        },
    };
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: group_rect,
            color: None,
        },
    );
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node a")
        .parent
        .replace(group_id);
    graph_value
        .nodes
        .get_mut(&b)
        .expect("node b")
        .parent
        .replace(group_id);

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
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view);

    let snapshot = canvas.sync_view_state(&mut host);
    let (base_geom, base_index) = canvas.canvas_derived(&host, &snapshot);

    let base_node_rect = base_geom.nodes.get(&a).expect("node rect").rect;
    let base_port = base_geom.port_center(a_out).expect("port center");

    let delta = CanvasPoint {
        x: 1000.0,
        y: 700.0,
    };
    let start_a = graph
        .read_ref(&host, |g| g.nodes.get(&a).map(|n| n.pos))
        .unwrap()
        .unwrap();
    let start_b = graph
        .read_ref(&host, |g| g.nodes.get(&b).map(|n| n.pos))
        .unwrap()
        .unwrap();
    let current_a = CanvasPoint {
        x: start_a.x + delta.x,
        y: start_a.y + delta.y,
    };
    let current_b = CanvasPoint {
        x: start_b.x + delta.x,
        y: start_b.y + delta.y,
    };

    let current_group_rect = CanvasRect {
        origin: CanvasPoint {
            x: group_rect.origin.x + delta.x,
            y: group_rect.origin.y + delta.y,
        },
        size: group_rect.size,
    };
    canvas.interaction.group_drag = Some(GroupDrag {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: group_rect,
        nodes: vec![(a, start_a), (b, start_b)],
        current_rect: current_group_rect,
        current_nodes: vec![(a, current_a), (b, current_b)],
        preview_rev: 1,
    });

    let got_group_rect = canvas.group_rect_with_preview(group_id, group_rect);
    assert_eq!(got_group_rect, current_group_rect);

    let (preview_geom, preview_index) = canvas.canvas_derived(&host, &snapshot);

    let preview_node_rect = preview_geom.nodes.get(&a).expect("node rect").rect;
    let node_origin = snapshot.interaction.node_origin.normalized();
    let size_canvas = crate::core::CanvasSize {
        width: base_node_rect.size.width.0,
        height: base_node_rect.size.height.0,
    };
    let expected_origin = crate::ui::canvas::geometry::node_rect_origin_from_anchor(
        current_a,
        size_canvas,
        node_origin,
    );
    assert_near(preview_node_rect.origin.x.0, expected_origin.x);
    assert_near(preview_node_rect.origin.y.0, expected_origin.y);

    let dx = expected_origin.x - base_node_rect.origin.x.0;
    let dy = expected_origin.y - base_node_rect.origin.y.0;
    let preview_port = preview_geom.port_center(a_out).expect("port center");
    assert_near(preview_port.x.0, base_port.x.0 + dx);
    assert_near(preview_port.y.0, base_port.y.0 + dy);

    let from_preview = preview_geom.port_center(a_out).expect("preview from");
    let to_preview = preview_geom.port_center(b_in).expect("preview to");
    let new_aabb = preview_index.edge_aabb(from_preview, to_preview, snapshot.zoom);
    let mut hits_new = Vec::new();
    preview_index.query_edges_in_rect(new_aabb, &mut hits_new);
    assert!(hits_new.contains(&edge_id));

    // Query a tiny rect around the old dragged endpoint; should not hit after a large move.
    let from_base = base_geom.port_center(a_out).expect("base from");
    let old_from_rect = Rect::new(
        Point::new(Px(from_base.x.0 - 1.0), Px(from_base.y.0 - 1.0)),
        Size::new(Px(2.0), Px(2.0)),
    );
    let mut hits_old = Vec::new();
    preview_index.query_edges_in_rect(old_from_rect, &mut hits_old);
    assert!(
        !hits_old.contains(&edge_id),
        "expected edge AABB to move with the dragged group"
    );

    // Sanity: base index still contains the edge at its original location.
    let from_base2 = base_geom.port_center(a_out).expect("base from");
    let to_base2 = base_geom.port_center(b_in).expect("base to");
    let base_aabb = base_index.edge_aabb(from_base2, to_base2, snapshot.zoom);
    let mut hits_base = Vec::new();
    base_index.query_edges_in_rect(base_aabb, &mut hits_base);
    assert!(hits_base.contains(&edge_id));
}

#[test]
fn group_drag_preview_cache_reuses_geometry_across_preview_rev_updates() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, _a_in, _a_out, b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);

    let group_id = crate::core::GroupId::new();
    let group_rect = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 800.0,
            height: 400.0,
        },
    };
    graph_value.groups.insert(
        group_id,
        crate::core::Group {
            title: "G".to_string(),
            rect: group_rect,
            color: None,
        },
    );
    graph_value
        .nodes
        .get_mut(&a)
        .expect("node a")
        .parent
        .replace(group_id);
    graph_value
        .nodes
        .get_mut(&b)
        .expect("node b")
        .parent
        .replace(group_id);

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph.clone(), view.clone());

    let snapshot0 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot0);

    let delta0 = CanvasPoint { x: 40.0, y: 25.0 };
    let start_a = graph
        .read_ref(&host, |g| g.nodes.get(&a).map(|n| n.pos))
        .unwrap()
        .unwrap();
    let start_b = graph
        .read_ref(&host, |g| g.nodes.get(&b).map(|n| n.pos))
        .unwrap()
        .unwrap();
    let pos0_a = CanvasPoint {
        x: start_a.x + delta0.x,
        y: start_a.y + delta0.y,
    };
    let pos0_b = CanvasPoint {
        x: start_b.x + delta0.x,
        y: start_b.y + delta0.y,
    };

    let rect0 = CanvasRect {
        origin: CanvasPoint {
            x: group_rect.origin.x + delta0.x,
            y: group_rect.origin.y + delta0.y,
        },
        size: group_rect.size,
    };
    canvas.interaction.group_drag = Some(GroupDrag {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: group_rect,
        nodes: vec![(a, start_a), (b, start_b)],
        current_rect: rect0,
        current_nodes: vec![(a, pos0_a), (b, pos0_b)],
        preview_rev: 1,
    });

    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);
    let geom0_ptr = Arc::as_ptr(&geom0) as usize;
    let index0_ptr = Arc::as_ptr(&index0) as usize;
    drop(geom0);
    drop(index0);

    // Bumping preview_rev without moving nodes should reuse cached preview geometry/index.
    {
        let drag = canvas.interaction.group_drag.as_mut().expect("group drag");
        drag.preview_rev = 2;
    }
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot0);
    let geom1_ptr = Arc::as_ptr(&geom1) as usize;
    let index1_ptr = Arc::as_ptr(&index1) as usize;
    assert_eq!(
        geom0_ptr, geom1_ptr,
        "expected group drag preview_rev bump to reuse cached preview geometry"
    );
    assert_eq!(
        index0_ptr, index1_ptr,
        "expected group drag preview_rev bump to reuse cached preview spatial index"
    );
    drop(geom1);
    drop(index1);

    // Moving nodes should update the preview in-place (still no full rebuild).
    let delta1 = CanvasPoint { x: 120.0, y: 90.0 };
    let pos1_a = CanvasPoint {
        x: start_a.x + delta1.x,
        y: start_a.y + delta1.y,
    };
    let pos1_b = CanvasPoint {
        x: start_b.x + delta1.x,
        y: start_b.y + delta1.y,
    };
    let rect1 = CanvasRect {
        origin: CanvasPoint {
            x: group_rect.origin.x + delta1.x,
            y: group_rect.origin.y + delta1.y,
        },
        size: group_rect.size,
    };
    {
        let drag = canvas.interaction.group_drag.as_mut().expect("group drag");
        drag.current_rect = rect1;
        drag.current_nodes = vec![(a, pos1_a), (b, pos1_b)];
        drag.preview_rev = 3;
    }
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot0);
    let geom2_ptr = Arc::as_ptr(&geom2) as usize;
    let index2_ptr = Arc::as_ptr(&index2) as usize;
    assert_eq!(
        geom1_ptr, geom2_ptr,
        "expected group drag preview movement to update cached preview geometry in-place"
    );
    assert_eq!(
        index1_ptr, index2_ptr,
        "expected group drag preview movement to update cached preview spatial index in-place"
    );
    drop(geom2);
    drop(index2);

    // If the base spatial index key changes, the preview cache must be invalidated and rebuilt.
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.spatial_index.edge_aabb_pad_screen_px = 200.0;
    });
    let snapshot1 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot1);

    let (geom3, index3) = canvas.canvas_derived(&host, &snapshot1);
    let geom3_ptr = Arc::as_ptr(&geom3) as usize;
    let index3_ptr = Arc::as_ptr(&index3) as usize;
    assert_ne!(
        geom2_ptr, geom3_ptr,
        "expected base index key change to invalidate and rebuild preview geometry"
    );
    assert_ne!(
        index2_ptr, index3_ptr,
        "expected base index key change to invalidate and rebuild preview spatial index"
    );
}

#[test]
fn group_resize_does_not_rebuild_canvas_derived_geometry() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    let base = CanvasRect {
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
            rect: base,
            color: None,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot = canvas.sync_view_state(&mut host);

    canvas.interaction.group_resize = Some(GroupResize {
        group: group_id,
        start_pos: Point::new(Px(0.0), Px(0.0)),
        start_rect: base,
        current_rect: base,
        preview_rev: 1,
    });

    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot);
    let geom0_ptr = Arc::as_ptr(&geom0) as usize;
    let index0_ptr = Arc::as_ptr(&index0) as usize;
    drop(geom0);
    drop(index0);

    // Even if the group resize preview rect changes, derived node/port geometry should stay cached.
    {
        let resize = canvas
            .interaction
            .group_resize
            .as_mut()
            .expect("group resize");
        resize.preview_rev = 2;
        resize.current_rect = CanvasRect {
            origin: CanvasPoint { x: 10.0, y: 20.0 },
            size: CanvasSize {
                width: 200.0,
                height: 180.0,
            },
        };
    }
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot);
    let geom1_ptr = Arc::as_ptr(&geom1) as usize;
    let index1_ptr = Arc::as_ptr(&index1) as usize;
    assert_eq!(
        geom0_ptr, geom1_ptr,
        "expected group resize preview to not rebuild derived node geometry"
    );
    assert_eq!(
        index0_ptr, index1_ptr,
        "expected group resize preview to not rebuild derived spatial index"
    );
}

#[test]
fn group_resize_clamps_to_children() {
    let mut host = TestUiHostImpl::default();
    let mut graph_value = Graph::new(GraphId::new());

    let group_id = crate::core::GroupId::new();
    let start_rect = CanvasRect {
        origin: CanvasPoint { x: 0.0, y: 0.0 },
        size: CanvasSize {
            width: 500.0,
            height: 500.0,
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

    // Child node placed so its right/bottom touch the start rect.
    let node_id = crate::core::NodeId::new();
    let node_size_px = CanvasSize {
        width: 200.0,
        height: 100.0,
    };
    graph_value.nodes.insert(
        node_id,
        Node {
            kind: crate::core::NodeKindKey::new("test.node"),
            kind_version: 1,
            pos: CanvasPoint { x: 300.0, y: 400.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: Some(group_id),
            extent: None,
            expand_parent: None,
            size: Some(node_size_px),
            hidden: false,
            collapsed: false,
            ports: Vec::new(),
            data: serde_json::Value::Null,
        },
    );

    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);
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

    assert!(super::super::group_resize::handle_group_resize_move(
        &mut canvas,
        &mut cx,
        &snapshot,
        Point::new(Px(-1000.0), Px(-1000.0)),
        Modifiers::default(),
        snapshot.zoom,
    ));

    let resize = canvas
        .interaction
        .group_resize
        .as_ref()
        .expect("group resize active");

    // Group must still contain its child; at zoom=1 the node rect uses size directly.
    let z = snapshot.zoom.max(1.0e-6);
    let node_w_canvas = node_size_px.width / z;
    let node_h_canvas = node_size_px.height / z;
    assert!(resize.current_rect.size.width + 1.0e-3 >= 300.0 + node_w_canvas);
    assert!(resize.current_rect.size.height + 1.0e-3 >= 400.0 + node_h_canvas);
}
