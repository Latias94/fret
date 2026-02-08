use std::sync::Arc;

use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::edge_types::NodeGraphEdgeTypes;
use crate::ui::presenter::{EdgeMarker, EdgeRenderHint, EdgeRouteKind, NodeGraphPresenter};
use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::prelude::{cubic_bezier_derivative, wire_ctrl_points};
use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

#[derive(Default)]
struct CaptureServices {
    path_prepares: Vec<(Vec<PathCommand>, PathStyle, PathConstraints)>,
}

impl fret_core::TextService for CaptureServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: fret_core::TextConstraints,
    ) -> (TextBlobId, fret_core::TextMetrics) {
        (
            TextBlobId::default(),
            fret_core::TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for CaptureServices {
    fn prepare(
        &mut self,
        commands: &[PathCommand],
        style: PathStyle,
        constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        self.path_prepares
            .push((commands.to_vec(), style, constraints));
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
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
) {
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
}

fn arrow_tip_and_base(cmds: &[PathCommand]) -> Option<(Point, Point)> {
    if cmds.len() != 4 {
        return None;
    }
    let PathCommand::MoveTo(tip) = cmds[0] else {
        return None;
    };
    let PathCommand::LineTo(p1) = cmds[1] else {
        return None;
    };
    let PathCommand::LineTo(p2) = cmds[2] else {
        return None;
    };
    if cmds[3] != PathCommand::Close {
        return None;
    }

    let base = Point::new(Px(0.5 * (p1.x.0 + p2.x.0)), Px(0.5 * (p1.y.0 + p2.y.0)));
    Some((tip, base))
}

fn sin_theta(a: Point, b: Point) -> f32 {
    let ax = a.x.0;
    let ay = a.y.0;
    let bx = b.x.0;
    let by = b.y.0;
    let al = (ax * ax + ay * ay).sqrt().max(1.0e-6);
    let bl = (bx * bx + by * by).sqrt().max(1.0e-6);
    ((ax * by - ay * bx).abs() / (al * bl)).max(0.0)
}

struct BezierRoutePresenter;

impl NodeGraphPresenter for BezierRoutePresenter {
    fn node_title(&self, _graph: &crate::core::Graph, _node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn port_label(&self, _graph: &crate::core::Graph, _port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from("")
    }

    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: crate::core::EdgeId,
        _style: &crate::ui::style::NodeGraphStyle,
    ) -> EdgeRenderHint {
        EdgeRenderHint {
            route: EdgeRouteKind::Bezier,
            ..EdgeRenderHint::default()
        }
    }
}

#[test]
fn bezier_markers_align_with_bezier_start_end_tangents() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (mut graph_value, _a, _a_in, a_out, b, b_in) = make_test_graph_two_nodes_with_ports();
    if let Some(node) = graph_value.nodes.get_mut(&b) {
        node.pos.x = 420.0;
        node.pos.y = 260.0;
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
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut style = NodeGraphStyle::default();
    style.node_width = 200.0;
    style.pin_radius = 6.0;

    let edge_types = NodeGraphEdgeTypes::new().with_fallback(|_g, _e, _style, mut h| {
        h.start_marker = Some(EdgeMarker::arrow(12.0));
        h.end_marker = Some(EdgeMarker::arrow(12.0));
        h
    });

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_presenter(BezierRoutePresenter)
        .with_edge_types(edge_types)
        .with_style(style);

    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let (c1, c2) = wire_ctrl_points(from, to, snapshot.zoom);
    let start_tangent = cubic_bezier_derivative(from, c1, c2, to, 0.0);
    let end_tangent = cubic_bezier_derivative(from, c1, c2, to, 1.0);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let mut arrows: Vec<(Point, Point)> = services
        .path_prepares
        .iter()
        .filter_map(|(cmds, style, _constraints)| {
            if matches!(style, PathStyle::Fill(_)) {
                arrow_tip_and_base(cmds)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(arrows.len(), 2, "expected start+end marker arrows");

    arrows.sort_by(|(t0, _), (t1, _)| {
        let d0 = (t0.x.0 - from.x.0).hypot(t0.y.0 - from.y.0);
        let d1 = (t1.x.0 - from.x.0).hypot(t1.y.0 - from.y.0);
        d0.total_cmp(&d1)
    });
    let (start_tip, start_base) = arrows[0];
    let (end_tip, end_base) = arrows[1];

    let start_axis = Point::new(
        Px(start_base.x.0 - start_tip.x.0),
        Px(start_base.y.0 - start_tip.y.0),
    );
    let end_axis = Point::new(
        Px(end_base.x.0 - end_tip.x.0),
        Px(end_base.y.0 - end_tip.y.0),
    );

    assert!(
        sin_theta(start_axis, start_tangent) <= 1.0e-3,
        "expected start marker to align with bezier start tangent"
    );
    assert!(
        sin_theta(end_axis, end_tangent) <= 1.0e-3,
        "expected end marker to align (colinear) with bezier end tangent"
    );

    let start_dot = start_axis.x.0 * start_tangent.x.0 + start_axis.y.0 * start_tangent.y.0;
    let end_dot = end_axis.x.0 * end_tangent.x.0 + end_axis.y.0 * end_tangent.y.0;
    assert!(
        start_dot > 0.0 && end_dot < 0.0,
        "expected start axis along +tangent and end axis along -tangent; start_dot={start_dot} end_dot={end_dot}"
    );
}
