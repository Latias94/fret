use std::sync::Arc;

use fret_core::{
    NodeId as UiNodeId, Point, Px, Rect, Scene, SceneOp, Size, TextBlobId, Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::io::NodeGraphViewState;
use crate::ui::presenter::{EdgeRenderHint, EdgeRouteKind, NodeGraphPresenter};
use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::{TestUiHostImpl, make_test_graph_two_nodes_with_ports};

#[derive(Default)]
struct CaptureServices;

impl fret_core::TextService for CaptureServices {
    fn prepare(
        &mut self,
        input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        let text = input.text();
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(text.len() as f32 * 7.0), Px(14.0)),
                baseline: Px(11.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for CaptureServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for CaptureServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        true
    }
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CaptureServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree,
        node: UiNodeId::default(),
        window: None,
        focus: None,
        children: &[],
        bounds,
        scale_factor: 1.0,
        accumulated_transform: Transform2D::IDENTITY,
        children_render_transform: None,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
        scene: &mut scene,
    };

    canvas.paint(&mut cx);
    scene
}

fn extract_label_text_origin(scene: &Scene, style: &NodeGraphStyle) -> Option<Point> {
    let ops = scene.ops();
    for ix in 0..ops.len().saturating_sub(1) {
        let SceneOp::Quad {
            order,
            background,
            border_color,
            ..
        } = ops[ix]
        else {
            continue;
        };
        if order != fret_core::DrawOrder(2) {
            continue;
        }
        if background != style.edge_label_background || border_color != style.edge_label_border {
            continue;
        }
        let SceneOp::Text { origin, .. } = ops[ix + 1] else {
            continue;
        };
        return Some(origin);
    }
    None
}

fn normal_from_tangent(tangent: Point) -> Point {
    let dx = tangent.x.0;
    let dy = tangent.y.0;
    let len = (dx * dx + dy * dy).sqrt();
    if !len.is_finite() || len <= 1.0e-6 {
        return Point::new(Px(0.0), Px(-1.0));
    }
    Point::new(Px(-dy / len), Px(dx / len))
}

struct LabelRoutePresenter {
    label: Arc<str>,
    route: EdgeRouteKind,
}

impl NodeGraphPresenter for LabelRoutePresenter {
    fn node_title(&self, _graph: &crate::core::Graph, _node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn port_label(&self, _graph: &crate::core::Graph, _port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
    ) -> EdgeRenderHint {
        EdgeRenderHint {
            label: Some(Arc::clone(&self.label)),
            route: self.route,
            width_mul: 1.0,
            ..EdgeRenderHint::default()
        }
    }
}

fn expected_text_origin_straight(
    style: &NodeGraphStyle,
    from: Point,
    to: Point,
    zoom: f32,
    label: &str,
) -> Point {
    let pos = Point::new(Px(0.5 * (from.x.0 + to.x.0)), Px(0.5 * (from.y.0 + to.y.0)));
    let d = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
    let normal = normal_from_tangent(d);
    let z = zoom.max(1.0e-6);
    let off = style.edge_label_offset / z;
    let anchor = Point::new(
        Px(pos.x.0 + normal.x.0 * off),
        Px(pos.y.0 + normal.y.0 * off),
    );

    let w = label.len() as f32 * 7.0;
    let h = 14.0;
    let baseline = 11.0;
    Point::new(
        Px(anchor.x.0 - 0.5 * w),
        Px(anchor.y.0 - 0.5 * h + baseline),
    )
}

fn expected_text_origin_step(
    style: &NodeGraphStyle,
    from: Point,
    to: Point,
    zoom: f32,
    label: &str,
) -> Point {
    let mx = 0.5 * (from.x.0 + to.x.0);
    let pos = Point::new(Px(mx), Px(0.5 * (from.y.0 + to.y.0)));
    let z = zoom.max(1.0e-6);
    let off = style.edge_label_offset / z;
    let anchor = Point::new(Px(pos.x.0), Px(pos.y.0 - off));

    let w = label.len() as f32 * 7.0;
    let h = 14.0;
    let baseline = 11.0;
    Point::new(
        Px(anchor.x.0 - 0.5 * w),
        Px(anchor.y.0 - 0.5 * h + baseline),
    )
}

fn capture_label_origin_for_route(route: EdgeRouteKind, zoom: f32) -> (Point, Point) {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    if let Some(node) = graph_value.nodes.get_mut(&b) {
        node.pos.y = 120.0;
    }

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
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
        s.interaction.bezier_hit_test_steps = 8;
    });

    let label: Arc<str> = Arc::<str>::from("EdgeLabel");
    let presenter = LabelRoutePresenter {
        label: Arc::clone(&label),
        route,
    };
    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(presenter);

    // Use non-default tokens so the test catches any accidental hard-coded constants.
    canvas.style.edge_label_offset = 37.0;
    canvas.style.edge_label_padding = 9.0;
    canvas.style.edge_label_corner_radius = 13.0;
    canvas.style.edge_label_border_width = 2.0;
    canvas.style.edge_label_max_width = 300.0;

    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();

    let mut origin: Option<Point> = None;
    for _ in 0..12 {
        let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
        origin = extract_label_text_origin(&scene, &canvas.style);
        if origin.is_some() {
            break;
        }
    }
    let origin = origin.expect("expected edge label to be painted");

    let expected = match route {
        EdgeRouteKind::Straight => {
            expected_text_origin_straight(&canvas.style, from, to, zoom, label.as_ref())
        }
        EdgeRouteKind::Step => {
            expected_text_origin_step(&canvas.style, from, to, zoom, label.as_ref())
        }
        EdgeRouteKind::Bezier => unreachable!("this helper only supports Straight/Step"),
    };

    (origin, expected)
}

#[test]
fn edge_label_anchor_matches_straight_route_math() {
    let (origin, expected) = capture_label_origin_for_route(EdgeRouteKind::Straight, 1.0);
    let dx = (origin.x.0 - expected.x.0).abs();
    let dy = (origin.y.0 - expected.y.0).abs();
    assert!(
        dx + dy <= 1.0e-3,
        "expected straight-route label origin to match midpoint+normal anchor; dx={dx} dy={dy}"
    );
}

#[test]
fn edge_label_anchor_matches_step_route_math() {
    let (origin, expected) = capture_label_origin_for_route(EdgeRouteKind::Step, 1.0);
    let dx = (origin.x.0 - expected.x.0).abs();
    let dy = (origin.y.0 - expected.y.0).abs();
    assert!(
        dx + dy <= 1.0e-3,
        "expected step-route label origin to match midpoint+up-normal anchor; dx={dx} dy={dy}"
    );
}

#[test]
fn edge_label_anchor_offset_is_zoom_safe_for_straight_route() {
    let (origin_z1, expected_z1) = capture_label_origin_for_route(EdgeRouteKind::Straight, 1.0);
    let (origin_z2, expected_z2) = capture_label_origin_for_route(EdgeRouteKind::Straight, 2.0);

    let err1 = (origin_z1.x.0 - expected_z1.x.0).abs() + (origin_z1.y.0 - expected_z1.y.0).abs();
    let err2 = (origin_z2.x.0 - expected_z2.x.0).abs() + (origin_z2.y.0 - expected_z2.y.0).abs();
    assert!(
        err1 <= 1.0e-3 && err2 <= 1.0e-3,
        "expected zoom-safe label anchors for straight route; err1={err1} err2={err2}"
    );
}

#[test]
fn edge_label_anchor_offset_is_zoom_safe_for_step_route() {
    let (origin_z1, expected_z1) = capture_label_origin_for_route(EdgeRouteKind::Step, 1.0);
    let (origin_z2, expected_z2) = capture_label_origin_for_route(EdgeRouteKind::Step, 2.0);

    let err1 = (origin_z1.x.0 - expected_z1.x.0).abs() + (origin_z1.y.0 - expected_z1.y.0).abs();
    let err2 = (origin_z2.x.0 - expected_z2.x.0).abs() + (origin_z2.y.0 - expected_z2.y.0).abs();
    assert!(
        err1 <= 1.0e-3 && err2 <= 1.0e-3,
        "expected zoom-safe label anchors for step route; err1={err1} err2={err2}"
    );
}
