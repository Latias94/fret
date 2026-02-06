use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphCanvas;
use crate::ui::edge_types::{EdgeCustomPath, EdgePathInput, NodeGraphEdgeTypes};
use crate::ui::presenter::EdgeMarker;

use super::prelude::path_start_end_tangents;
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

#[test]
fn custom_edge_marker_falls_back_to_from_to_tangent_when_path_has_no_tangents() {
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
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let edge_types = NodeGraphEdgeTypes::new()
        .with_fallback(|_g, _e, _style, mut h| {
            h.start_marker = Some(EdgeMarker::arrow(12.0));
            h.end_marker = Some(EdgeMarker::arrow(12.0));
            h
        })
        .with_fallback_path(|_g, _e, _style, _hint, input: EdgePathInput| {
            Some(EdgeCustomPath {
                cache_key: 1,
                commands: vec![PathCommand::MoveTo(input.from)],
            })
        });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);
    let snapshot = canvas.sync_view_state(&mut host);
    let geom = canvas.canvas_geometry(&host, &snapshot);
    let from = geom.port_center(a_out).expect("from port center");
    let to = geom.port_center(b_in).expect("to port center");

    let custom = canvas
        .graph
        .read_ref(&host, |g| {
            let hint = canvas.edge_render_hint(g, edge_id);
            canvas.edge_custom_path(g, edge_id, &hint, from, to, snapshot.zoom)
        })
        .ok()
        .flatten()
        .expect("custom path must exist");
    assert!(
        path_start_end_tangents(&custom.commands).is_none(),
        "commands must not provide tangents (this test expects fallback tangents)"
    );

    let dir = Point::new(Px(to.x.0 - from.x.0), Px(to.y.0 - from.y.0));
    let dir_len = (dir.x.0 * dir.x.0 + dir.y.0 * dir.y.0).sqrt().max(1.0e-6);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let arrows: Vec<(Point, Point)> = services
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
    assert_eq!(
        arrows.len(),
        2,
        "expected start+end marker arrow paths to be prepared"
    );

    let mut signs = Vec::new();
    for (tip, base) in arrows {
        let vx = base.x.0 - tip.x.0;
        let vy = base.y.0 - tip.y.0;
        let v_len = (vx * vx + vy * vy).sqrt().max(1.0e-6);

        let cross = (vx * dir.y.0 - vy * dir.x.0).abs();
        let sin_theta = cross / (v_len * dir_len);
        assert!(
            sin_theta <= 1.0e-3,
            "expected arrow axis to align with fallback tangent; sin(theta)={sin_theta}"
        );

        let dot = vx * dir.x.0 + vy * dir.y.0;
        signs.push(dot.total_cmp(&0.0));
    }

    assert!(
        signs.iter().any(|s| *s == std::cmp::Ordering::Greater)
            && signs.iter().any(|s| *s == std::cmp::Ordering::Less),
        "expected one marker to point along +dir and the other along -dir"
    );
}
