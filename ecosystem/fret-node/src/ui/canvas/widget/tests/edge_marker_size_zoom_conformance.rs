use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphCanvas;
use crate::ui::edge_types::NodeGraphEdgeTypes;
use crate::ui::presenter::EdgeMarker;

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

impl fret_core::MaterialService for CaptureServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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

fn capture_arrow_axis_lengths_for_zoom(zoom: f32) -> Vec<f32> {
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
        s.zoom = zoom;
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let edge_types = NodeGraphEdgeTypes::new().with_fallback(|_g, _e, _style, mut h| {
        h.start_marker = Some(EdgeMarker::arrow(12.0));
        h.end_marker = Some(EdgeMarker::arrow(12.0));
        h
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_edge_types(edge_types);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    services
        .path_prepares
        .iter()
        .filter_map(|(cmds, style, _constraints)| {
            if matches!(style, PathStyle::Fill(_)) {
                let (tip, base) = arrow_tip_and_base(cmds)?;
                let dx = base.x.0 - tip.x.0;
                let dy = base.y.0 - tip.y.0;
                Some((dx * dx + dy * dy).sqrt())
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn edge_marker_arrow_size_is_zoom_safe() {
    let len_z1 = capture_arrow_axis_lengths_for_zoom(1.0);
    let len_z2 = capture_arrow_axis_lengths_for_zoom(2.0);
    assert_eq!(len_z1.len(), 2, "expected start+end marker at zoom=1");
    assert_eq!(len_z2.len(), 2, "expected start+end marker at zoom=2");

    let avg = |v: &[f32]| -> f32 { v.iter().sum::<f32>() / v.len().max(1) as f32 };
    let a1 = avg(&len_z1);
    let a2 = avg(&len_z2);

    // Marker size is specified in screen-space pixels; in canvas-space it should shrink by 1/zoom.
    // With zoom=2, the axis length should be half.
    let err = (a1 - 2.0 * a2).abs();
    assert!(
        err <= 1.0e-3,
        "expected arrow axis length to scale as 1/zoom; a1={a1} a2={a2} err={err}"
    );

    let within = |v: &[f32]| -> f32 {
        let mn = v.iter().copied().fold(f32::INFINITY, f32::min);
        let mx = v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (mx - mn).abs()
    };
    assert!(
        within(&len_z1) <= 1.0e-3 && within(&len_z2) <= 1.0e-3,
        "expected start/end arrow marker sizes to match at each zoom"
    );
}
