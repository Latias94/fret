use fret_core::{Point, Px, Rect, Size};

use crate::io::NodeGraphViewState;
use crate::ui::presenter::EdgeRouteKind;

use super::*;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1200.0), Px(800.0)),
    )
}

fn unit_normal(from: Point, to: Point) -> (f32, f32) {
    let dx = to.x.0 - from.x.0;
    let dy = to.y.0 - from.y.0;
    let len = (dx * dx + dy * dy).sqrt().max(1.0e-6);
    (-dy / len, dx / len)
}

fn offset_point(p: Point, nx: f32, ny: f32, d: f32) -> Point {
    Point::new(Px(p.x.0 + nx * d), Px(p.y.0 + ny * d))
}

fn run_case(zoom: f32, edge_interaction_width: f32, wire_width: f32) -> (bool, bool) {
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
    let view = host.models.insert(NodeGraphViewState::default());
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = zoom;
        s.interaction.edge_interaction_width = edge_interaction_width;
    });

    let edge_types = crate::ui::NodeGraphEdgeTypes::new().with_fallback(|_g, _e, _style, mut h| {
        h.route = EdgeRouteKind::Straight;
        h
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);
    canvas.style.wire_width = wire_width;

    let mut services = NullServices::default();
    let mut prevented_default_actions = fret_runtime::DefaultActionSet::default();
    let cx = event_cx(
        &mut host,
        &mut services,
        bounds(),
        &mut prevented_default_actions,
    );

    let snapshot = canvas.sync_view_state(cx.app);
    assert!((snapshot.zoom - zoom).abs() <= 1.0e-6);

    let geom = canvas.canvas_geometry(&*cx.app, &snapshot);
    let index = canvas.geometry.index.clone();

    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");
    let mid = Point::new(Px((from.x.0 + to.x.0) * 0.5), Px((from.y.0 + to.y.0) * 0.5));
    let (nx, ny) = unit_normal(from, to);

    let hit_w_canvas = (edge_interaction_width.max(wire_width)) / zoom.max(1.0e-6);
    let pos_hit = offset_point(mid, nx, ny, hit_w_canvas * 0.9);
    let pos_miss = offset_point(mid, nx, ny, hit_w_canvas * 1.1);

    let graph_snapshot = canvas
        .graph
        .read_ref(cx.app, |g| g.clone())
        .ok()
        .unwrap_or_default();

    let mut scratch = super::super::HitTestScratch::default();
    let mut ctx = super::super::HitTestCtx::new(geom.as_ref(), &index, zoom, &mut scratch);
    let hit = canvas.hit_edge(&graph_snapshot, &snapshot, &mut ctx, pos_hit);
    let miss = canvas.hit_edge(&graph_snapshot, &snapshot, &mut ctx, pos_miss);

    (hit == Some(edge_id), miss == Some(edge_id))
}

#[test]
fn edge_hit_width_falls_back_to_wire_width_when_larger() {
    for zoom in [0.5, 1.0, 2.0] {
        let (hit, miss) = run_case(zoom, 0.0, 20.0);
        assert!(hit, "expected hit at zoom={zoom}");
        assert!(!miss, "expected miss at zoom={zoom}");
    }
}

#[test]
fn edge_hit_width_prefers_edge_interaction_width_when_larger() {
    for zoom in [0.5, 1.0, 2.0] {
        let (hit, miss) = run_case(zoom, 12.0, 2.0);
        assert!(hit, "expected hit at zoom={zoom}");
        assert!(!miss, "expected miss at zoom={zoom}");
    }
}
