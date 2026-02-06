use std::sync::Arc;

use fret_core::{PathCommand, Point, Px};

use crate::core::{Edge, EdgeId, EdgeKind};

use super::super::NodeGraphCanvas;
use super::super::path_midpoint_and_normal;
use super::super::{HitTestCtx, HitTestScratch};
use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

const BEND_SCREEN_PX: f32 = 2048.0;

#[test]
fn edge_types_updates_invalidate_spatial_index_and_hit_testing_uses_new_custom_paths() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();

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

    // Start with no `edgeTypes` (no custom paths).
    let mut canvas = NodeGraphCanvas::new(graph, view);
    let snapshot0 = canvas.sync_view_state(&mut host);
    let (geom0, index0) = canvas.canvas_derived(&host, &snapshot0);

    let zoom = snapshot0.zoom;
    let from = geom0.port_center(a_out).expect("from port center");
    let to = geom0.port_center(b_in).expect("to port center");

    // Create a candidate point on the would-be custom path that is intentionally far away from the
    // default conservative wire AABB.
    let bend = BEND_SCREEN_PX / zoom.max(1.0e-6);
    let c1 = Point::new(Px(from.x.0), Px(from.y.0 - bend));
    let c2 = Point::new(Px(to.x.0), Px(to.y.0 - bend));
    let commands = vec![
        PathCommand::MoveTo(from),
        PathCommand::CubicTo {
            ctrl1: c1,
            ctrl2: c2,
            to,
        },
    ];

    let steps = usize::from(snapshot0.interaction.bezier_hit_test_steps.max(1));
    let (mid, _normal) = path_midpoint_and_normal(&commands, steps).expect("midpoint must exist");

    let default_aabb = fret_canvas::wires::wire_aabb(from, to, zoom, 0.0);
    assert!(
        !default_aabb.contains(mid),
        "expected the custom-path midpoint to lie outside the default wire AABB (sanity)"
    );

    // Without `edgeTypes`, this point should not hit the edge (both broad-phase and narrow-phase).
    let mut candidates = Vec::new();
    let candidates = index0.query_edges_sorted_dedup(mid, 1.0, &mut candidates);
    assert!(
        !candidates.contains(&edge_id),
        "expected the spatial index to exclude far-away points when no custom edge path is present"
    );

    let graph_snapshot = canvas
        .graph
        .read_ref(&host, |g| g.clone())
        .unwrap_or_default();
    let mut scratch = HitTestScratch::default();
    let mut ctx = HitTestCtx::new(&geom0, &index0, zoom, &mut scratch);
    assert_eq!(
        canvas.hit_edge(&graph_snapshot, &snapshot0, &mut ctx, mid),
        None,
        "expected no edge hit before edgeTypes provides a custom path"
    );

    // Now attach `edgeTypes` with a Stage 2 custom path matching the commands above.
    let edge_types = crate::ui::NodeGraphEdgeTypes::new().register_path(
        crate::ui::EdgeTypeKey::new("data"),
        |_graph, _edge, _style, _hint, input| {
            let z = input.zoom.max(1.0e-6);
            let bend = BEND_SCREEN_PX / z;
            let c1 = Point::new(Px(input.from.x.0), Px(input.from.y.0 - bend));
            let c2 = Point::new(Px(input.to.x.0), Px(input.to.y.0 - bend));
            Some(crate::ui::EdgeCustomPath {
                cache_key: 1,
                commands: vec![
                    PathCommand::MoveTo(input.from),
                    PathCommand::CubicTo {
                        ctrl1: c1,
                        ctrl2: c2,
                        to: input.to,
                    },
                ],
            })
        },
    );
    canvas.edge_types = Some(edge_types);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    assert!(
        !Arc::ptr_eq(&index0, &index1),
        "expected edgeTypes updates to invalidate the spatial index"
    );

    let mut candidates = Vec::new();
    let candidates = index1.query_edges_sorted_dedup(mid, 1.0, &mut candidates);
    assert!(
        candidates.contains(&edge_id),
        "expected the spatial index to include the custom edge path bounds after edgeTypes updates"
    );

    let mut scratch = HitTestScratch::default();
    let mut ctx = HitTestCtx::new(&geom1, &index1, snapshot1.zoom, &mut scratch);
    assert_eq!(
        canvas.hit_edge(&graph_snapshot, &snapshot1, &mut ctx, mid),
        Some(edge_id),
        "expected hit-testing to use the new custom edge path after edgeTypes updates"
    );
}
