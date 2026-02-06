use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind};

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports_spaced_x};

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
}

#[test]
fn node_resize_preview_cache_reuses_geometry_across_preview_rev_updates() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, _a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    // Ensure base geometry + spatial index caches exist (resize previews are keyed off base_index_key).
    let snapshot0 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot0);

    let pos0 = CanvasPoint { x: 600.0, y: 300.0 };
    let size0_px = CanvasSize {
        width: 500.0,
        height: 200.0,
    };

    let (geom0, index0) = canvas
        .node_resize_preview_derived(&host, &snapshot0, 1, a, pos0, Some(size0_px))
        .expect("preview");
    let geom0_ptr = Arc::as_ptr(&geom0) as usize;
    let index0_ptr = Arc::as_ptr(&index0) as usize;
    drop(geom0);
    drop(index0);

    // Bumping preview_rev without changing rect should not rebuild preview geometry/index.
    let (geom1, index1) = canvas
        .node_resize_preview_derived(&host, &snapshot0, 2, a, pos0, Some(size0_px))
        .expect("preview");
    let geom1_ptr = Arc::as_ptr(&geom1) as usize;
    let index1_ptr = Arc::as_ptr(&index1) as usize;
    assert_eq!(
        geom0_ptr, geom1_ptr,
        "expected preview_rev bump with no rect change to reuse cached preview geometry"
    );
    assert_eq!(
        index0_ptr, index1_ptr,
        "expected preview_rev bump with no rect change to reuse cached preview spatial index"
    );
    drop(geom1);
    drop(index1);

    // Changing position and size should update the preview in-place (still no full rebuild).
    let pos1 = CanvasPoint { x: 800.0, y: 420.0 };
    let size1_px = CanvasSize {
        width: 900.0,
        height: 240.0,
    };
    let (geom2, index2) = canvas
        .node_resize_preview_derived(&host, &snapshot0, 3, a, pos1, Some(size1_px))
        .expect("preview");
    let geom2_ptr = Arc::as_ptr(&geom2) as usize;
    let index2_ptr = Arc::as_ptr(&index2) as usize;
    assert_eq!(
        geom1_ptr, geom2_ptr,
        "expected resize preview updates to mutate cached preview geometry in-place"
    );
    assert_eq!(
        index1_ptr, index2_ptr,
        "expected resize preview updates to mutate cached preview spatial index in-place"
    );
    drop(geom2);
    drop(index2);

    // If the base spatial index key changes, the preview cache must be invalidated and rebuilt.
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.spatial_index.edge_aabb_pad_screen_px = 200.0;
    });
    let snapshot1 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot1);

    let (geom3, index3) = canvas
        .node_resize_preview_derived(&host, &snapshot1, 4, a, pos1, Some(size1_px))
        .expect("preview");
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
fn node_resize_preview_updates_node_rect_ports_and_edge_index() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
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
    let view = insert_view(&mut host);
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let snapshot = canvas.sync_view_state(&mut host);
    let (base_geom, _base_index) = canvas.canvas_derived(&host, &snapshot);

    let _base_node_rect = base_geom.nodes.get(&a).expect("node rect").rect;
    let base_out_center = base_geom.port_center(a_out).expect("out port center");

    let next_pos = CanvasPoint { x: 900.0, y: 400.0 };
    let next_size_px = CanvasSize {
        width: 800.0,
        height: 300.0,
    };

    let (preview_geom, preview_index) = canvas
        .node_resize_preview_derived(&host, &snapshot, 1, a, next_pos, Some(next_size_px))
        .expect("preview");

    let preview_node_rect = preview_geom.nodes.get(&a).expect("node rect").rect;
    assert_near(
        preview_node_rect.size.width.0 * snapshot.zoom,
        next_size_px.width,
    );
    assert_near(
        preview_node_rect.size.height.0 * snapshot.zoom,
        next_size_px.height,
    );

    let node_origin = snapshot.interaction.node_origin.normalized();
    let size_canvas = crate::core::CanvasSize {
        width: preview_node_rect.size.width.0,
        height: preview_node_rect.size.height.0,
    };
    let expected_origin = crate::ui::canvas::geometry::node_rect_origin_from_anchor(
        next_pos,
        size_canvas,
        node_origin,
    );
    assert_near(preview_node_rect.origin.x.0, expected_origin.x);
    assert_near(preview_node_rect.origin.y.0, expected_origin.y);

    let preview_out_center = preview_geom.port_center(a_out).expect("out port center");
    assert_near(
        preview_out_center.x.0,
        preview_node_rect.origin.x.0 + preview_node_rect.size.width.0,
    );

    let old_out_rect = Rect::new(
        Point::new(Px(base_out_center.x.0 - 1.0), Px(base_out_center.y.0 - 1.0)),
        Size::new(Px(2.0), Px(2.0)),
    );
    let mut hits_old = Vec::new();
    preview_index.query_edges_in_rect(old_out_rect, &mut hits_old);
    assert!(
        !hits_old.contains(&edge_id),
        "expected edge index to move with resized node"
    );

    let from_preview = preview_geom.port_center(a_out).expect("preview from");
    let to_preview = preview_geom.port_center(b_in).expect("preview to");
    let aabb = preview_index.edge_aabb(from_preview, to_preview, snapshot.zoom);
    let mut hits_new = Vec::new();
    preview_index.query_edges_in_rect(aabb, &mut hits_new);
    assert!(hits_new.contains(&edge_id));
}

#[test]
fn node_resize_preview_rev_updates_do_not_drift() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, a_out, _b, _b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let snapshot = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot);

    let pos1 = CanvasPoint { x: 400.0, y: 100.0 };
    let size1_px = CanvasSize {
        width: 500.0,
        height: 200.0,
    };
    let _ = canvas
        .node_resize_preview_derived(&host, &snapshot, 1, a, pos1, Some(size1_px))
        .expect("preview1");

    let pos2 = CanvasPoint {
        x: 1000.0,
        y: 700.0,
    };
    let size2_px = CanvasSize {
        width: 900.0,
        height: 240.0,
    };
    let (preview2_geom, _preview2_index) = canvas
        .node_resize_preview_derived(&host, &snapshot, 2, a, pos2, Some(size2_px))
        .expect("preview2");

    let rect = preview2_geom.nodes.get(&a).expect("node rect").rect;
    assert_near(rect.size.width.0 * snapshot.zoom, size2_px.width);
    assert_near(rect.size.height.0 * snapshot.zoom, size2_px.height);

    let out_center = preview2_geom.port_center(a_out).expect("out port center");
    assert_near(out_center.x.0, rect.origin.x.0 + rect.size.width.0);
}
