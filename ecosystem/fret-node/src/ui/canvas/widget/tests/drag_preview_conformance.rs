use std::sync::Arc;

use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
}

#[test]
fn drag_preview_cache_reuses_geometry_across_preview_rev_updates() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(500.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    // Ensure base geometry + spatial index caches exist (drag previews are keyed off base_index_key).
    let snapshot0 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot0);

    let pos0 = crate::core::CanvasPoint { x: 600.0, y: 300.0 };
    let pos1 = crate::core::CanvasPoint { x: 800.0, y: 420.0 };

    let (geom0, index0) = canvas
        .drag_preview_derived(
            &host,
            &snapshot0,
            super::super::DragPreviewKind::NodeDrag,
            1,
            &[(a, pos0)],
        )
        .expect("preview");
    let geom0_ptr = Arc::as_ptr(&geom0) as usize;
    let index0_ptr = Arc::as_ptr(&index0) as usize;
    drop(geom0);
    drop(index0);

    // Bumping preview_rev without moving nodes should not rebuild preview geometry/index.
    let (geom1, index1) = canvas
        .drag_preview_derived(
            &host,
            &snapshot0,
            super::super::DragPreviewKind::NodeDrag,
            2,
            &[(a, pos0)],
        )
        .expect("preview");
    let geom1_ptr = Arc::as_ptr(&geom1) as usize;
    let index1_ptr = Arc::as_ptr(&index1) as usize;
    assert_eq!(
        geom0_ptr, geom1_ptr,
        "expected preview_rev bump with no movement to reuse cached preview geometry"
    );
    assert_eq!(
        index0_ptr, index1_ptr,
        "expected preview_rev bump with no movement to reuse cached preview spatial index"
    );
    drop(geom1);
    drop(index1);

    // Moving nodes should update the preview in-place (still no full rebuild).
    let (geom2, index2) = canvas
        .drag_preview_derived(
            &host,
            &snapshot0,
            super::super::DragPreviewKind::NodeDrag,
            3,
            &[(a, pos1)],
        )
        .expect("preview");
    let geom2_ptr = Arc::as_ptr(&geom2) as usize;
    let index2_ptr = Arc::as_ptr(&index2) as usize;
    assert_eq!(
        geom1_ptr, geom2_ptr,
        "expected drag preview movement to update cached preview geometry in-place"
    );
    assert_eq!(
        index1_ptr, index2_ptr,
        "expected drag preview movement to update cached preview spatial index in-place"
    );

    // Sanity: the moved node's output port remains hittable in the preview geometry.
    let _ = geom2.port_center(a_out).expect("preview port center");
    let _ = geom2.port_center(b_in).expect("preview port center");
    let geom2_keep = geom2.clone();
    let index2_keep = index2.clone();
    drop(geom2);
    drop(index2);

    // If the base spatial index key changes, the preview cache must be invalidated and rebuilt.
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.spatial_index.edge_aabb_pad_screen_px = 200.0;
    });
    let snapshot1 = canvas.sync_view_state(&mut host);
    let _ = canvas.canvas_derived(&host, &snapshot1);

    let (geom3, index3) = canvas
        .drag_preview_derived(
            &host,
            &snapshot1,
            super::super::DragPreviewKind::NodeDrag,
            4,
            &[(a, pos1)],
        )
        .expect("preview");
    assert!(
        !Arc::ptr_eq(&geom2_keep, &geom3),
        "expected base index key change to invalidate and rebuild preview geometry"
    );
    assert!(
        !Arc::ptr_eq(&index2_keep, &index3),
        "expected base index key change to invalidate and rebuild preview spatial index"
    );
}

#[test]
fn drag_preview_updates_node_rect_port_centers_and_edge_index() {
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
    let view = host.models.insert(NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view);

    let snapshot = canvas.sync_view_state(&mut host);
    let (base_geom, base_index) = canvas.canvas_derived(&host, &snapshot);

    let base_node_rect = base_geom.nodes.get(&a).expect("node rect").rect;
    let base_port = base_geom.port_center(a_out).expect("port center");

    let next_pos = CanvasPoint {
        x: 1000.0,
        y: 400.0,
    };
    let (preview_geom, preview_index) = canvas
        .drag_preview_derived(
            &host,
            &snapshot,
            super::super::DragPreviewKind::NodeDrag,
            1,
            &[(a, next_pos)],
        )
        .expect("preview");

    let preview_node_rect = preview_geom.nodes.get(&a).expect("node rect").rect;
    let preview_port = preview_geom.port_center(a_out).expect("port center");

    let node_origin = snapshot.interaction.node_origin.normalized();
    let size_canvas = crate::core::CanvasSize {
        width: base_node_rect.size.width.0,
        height: base_node_rect.size.height.0,
    };
    let expected_origin = crate::ui::canvas::geometry::node_rect_origin_from_anchor(
        next_pos,
        size_canvas,
        node_origin,
    );
    assert_near(preview_node_rect.origin.x.0, expected_origin.x);
    assert_near(preview_node_rect.origin.y.0, expected_origin.y);

    let dx = expected_origin.x - base_node_rect.origin.x.0;
    let dy = expected_origin.y - base_node_rect.origin.y.0;
    assert_near(preview_port.x.0, base_port.x.0 + dx);
    assert_near(preview_port.y.0, base_port.y.0 + dy);

    let from_base = base_geom.port_center(a_out).expect("base from");
    let to_base = base_geom.port_center(b_in).expect("base to");
    let _old_aabb = base_index.edge_aabb(from_base, to_base, snapshot.zoom);

    let from_preview = preview_geom.port_center(a_out).expect("preview from");
    let to_preview = preview_geom.port_center(b_in).expect("preview to");
    let new_aabb = preview_index.edge_aabb(from_preview, to_preview, snapshot.zoom);

    let mut hits_new = Vec::new();
    preview_index.query_edges_in_rect(new_aabb, &mut hits_new);
    assert!(hits_new.contains(&edge_id));

    let mut hits_old = Vec::new();
    let old_from_rect = Rect::new(
        Point::new(Px(from_base.x.0 - 1.0), Px(from_base.y.0 - 1.0)),
        Size::new(Px(2.0), Px(2.0)),
    );
    preview_index.query_edges_in_rect(old_from_rect, &mut hits_old);
    assert!(
        !hits_old.contains(&edge_id),
        "expected edge AABB to move with the dragged endpoint"
    );

    // Sanity: preview should still return the edge when querying a large rect.
    let mut hits_all = Vec::new();
    preview_index.query_edges_in_rect(
        Rect::new(
            Point::new(Px(-10_000.0), Px(-10_000.0)),
            Size::new(Px(20_000.0), Px(20_000.0)),
        ),
        &mut hits_all,
    );
    assert!(hits_all.contains(&edge_id));
}
