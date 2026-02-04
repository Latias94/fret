use fret_core::{Point, Px, Rect, Size};

use crate::core::{CanvasPoint, Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;

use super::super::NodeGraphCanvas;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn assert_near(a: f32, b: f32) {
    assert!((a - b).abs() <= 1.0e-5, "{a} != {b}");
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
