use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, CanvasSize, Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
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
    let view = host.models.insert(NodeGraphViewState::default());
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
    let view = host.models.insert(NodeGraphViewState::default());
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
