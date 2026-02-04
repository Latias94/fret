use fret_core::{Point, Px};
use uuid::Uuid;

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;

use super::prelude::*;
use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports_spaced_x};

fn pan_for_canvas_point_at_window_point(
    canvas_point: Point,
    window_point: Point,
    zoom: f32,
) -> crate::core::CanvasPoint {
    let z = zoom.max(1.0e-6);
    crate::core::CanvasPoint {
        x: window_point.x.0 / z - canvas_point.x.0,
        y: window_point.y.0 / z - canvas_point.y.0,
    }
}

fn canvas_from_window_point(
    window_point: Point,
    pan: crate::core::CanvasPoint,
    zoom: f32,
) -> Point {
    let z = zoom.max(1.0e-6);
    Point::new(
        Px(window_point.x.0 / z - pan.x),
        Px(window_point.y.0 / z - pan.y),
    )
}

fn hit_edge_at(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    snapshot: &crate::ui::canvas::state::ViewSnapshot,
    pos: Point,
) -> Option<crate::core::EdgeId> {
    let (geom, index) = canvas.canvas_derived(&*host, snapshot);
    let this = canvas;
    this.graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx =
                HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);
            this.hit_edge(graph, snapshot, &mut ctx, pos)
        })
        .ok()
        .flatten()
}

#[test]
fn port_hit_testing_is_screen_px_invariant_under_semantic_zoom() {
    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _a_in, _a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view.clone());

    let anchor_window = Point::new(Px(300.0), Px(200.0));
    let r_px = canvas.style.pin_radius.max(0.0);

    // Test a spread of zoom levels; the port visual radius stays constant in window space
    // under semantic zoom.
    for zoom in [0.5, 1.0, 1.25, 2.0, 4.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
        });
        let snapshot = canvas.sync_view_state(&mut host);
        assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

        // Anchor the port center to a fixed window-space position by solving for pan.
        let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
        let handle = geom
            .ports
            .get(&b_in)
            .expect("target port handle should exist");
        let pan = pan_for_canvas_point_at_window_point(handle.center, anchor_window, zoom);
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = pan;
        });
        let snapshot = canvas.sync_view_state(&mut host);

        let inside_window = Point::new(Px(anchor_window.x.0 + (r_px - 1.0)), anchor_window.y);
        let outside_window = Point::new(Px(anchor_window.x.0 + (r_px + 1.0)), anchor_window.y);
        let inside = canvas_from_window_point(inside_window, pan, zoom);
        let outside = canvas_from_window_point(outside_window, pan, zoom);

        let (geom, index) = canvas.canvas_derived(&host, &snapshot);
        let mut scratch = HitTestScratch::default();
        let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), snapshot.zoom, &mut scratch);

        assert_eq!(
            canvas.hit_port(&mut ctx, inside),
            Some(b_in),
            "expected a point inside the port visual radius to hit across zoom={zoom}"
        );
        assert_eq!(
            canvas.hit_port(&mut ctx, outside),
            None,
            "expected a point outside the port visual radius to miss across zoom={zoom}"
        );
    }
}

#[test]
fn edge_hit_testing_is_screen_px_invariant_under_semantic_zoom() {
    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) =
        make_test_graph_two_nodes_with_ports_spaced_x(260.0);

    let edge_id = EdgeId(Uuid::from_u128(1));
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

    // Force a straight-line path so the screen-space distance assertions are stable.
    let edge_types =
        crate::ui::NodeGraphEdgeTypes::new().with_fallback_path(|_g, _e, _style, _hint, input| {
            Some(crate::ui::edge_types::EdgeCustomPath {
                cache_key: 1,
                commands: vec![
                    fret_core::PathCommand::MoveTo(input.from),
                    fret_core::PathCommand::LineTo(input.to),
                ],
            })
        });

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_edge_types(edge_types);

    let anchor_window = Point::new(Px(320.0), Px(260.0));

    // Ensure edge slop dominates wire width so the threshold is unambiguous.
    let edge_slop_px = 12.0_f32;
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.edge_interaction_width = edge_slop_px;
    });

    for zoom in [0.5, 1.0, 1.25, 2.0, 4.0] {
        let _ = view.update(&mut host, |s, _cx| {
            s.zoom = zoom;
        });
        let snapshot = canvas.sync_view_state(&mut host);
        assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

        let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
        let from = geom.port_center(a_out).expect("from port center");
        let to = geom.port_center(b_in).expect("to port center");
        let mid = Point::new(Px(0.5 * (from.x.0 + to.x.0)), Px(0.5 * (from.y.0 + to.y.0)));

        // Anchor the edge midpoint to a fixed window-space position by solving for pan.
        let pan = pan_for_canvas_point_at_window_point(mid, anchor_window, zoom);
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = pan;
        });
        let snapshot = canvas.sync_view_state(&mut host);

        let inside_window = Point::new(
            anchor_window.x,
            Px(anchor_window.y.0 + (edge_slop_px - 1.0)),
        );
        let outside_window = Point::new(
            anchor_window.x,
            Px(anchor_window.y.0 + (edge_slop_px + 1.0)),
        );
        let inside = canvas_from_window_point(inside_window, pan, zoom);
        let outside = canvas_from_window_point(outside_window, pan, zoom);

        assert_eq!(
            hit_edge_at(&mut canvas, &mut host, &snapshot, inside),
            Some(edge_id),
            "expected a point inside the edge screen slop to hit across zoom={zoom}"
        );
        assert_eq!(
            hit_edge_at(&mut canvas, &mut host, &snapshot, outside),
            None,
            "expected a point outside the edge screen slop to miss across zoom={zoom}"
        );
    }
}
