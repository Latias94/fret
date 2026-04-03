use fret_core::{NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle};
use fret_core::{Point, Px, Rect, Scene, Size, TextBlobId, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};
use std::sync::Arc;

use crate::core::{Edge, EdgeId, EdgeKind, Graph};
use crate::ui::NodeGraphCanvas;
use crate::ui::edge_types::NodeGraphEdgeTypes;
use crate::ui::presenter::EdgeMarker;
use crate::ui::skin::{EdgeChromeHint, NodeGraphSkin};

use super::{
    TestUiHostImpl, insert_graph_view_editor_config_with, make_test_graph_two_nodes_with_ports,
};

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

    let mut cx = fret_ui::retained_bridge::PaintCx::new(
        host,
        tree,
        UiNodeId::default(),
        None,
        None,
        &[],
        bounds,
        1.0,
        Transform2D::IDENTITY,
        None,
        services,
        &mut observe_model,
        &mut observe_global,
        &mut scene,
    );

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

fn capture_arrow_axis_lengths(services: &CaptureServices) -> Vec<f32> {
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

#[derive(Clone)]
struct MarkerChromeSkin {
    start: EdgeMarker,
    end: EdgeMarker,
}

impl NodeGraphSkin for MarkerChromeSkin {
    fn edge_chrome_hint(
        &self,
        _graph: &Graph,
        _edge: EdgeId,
        _style: &crate::ui::NodeGraphStyle,
        _selected: bool,
        _hovered: bool,
    ) -> EdgeChromeHint {
        EdgeChromeHint {
            start_marker: Some(self.start.clone()),
            end_marker: Some(self.end.clone()),
            ..EdgeChromeHint::default()
        }
    }
}

#[test]
fn skin_edge_chrome_hint_can_override_edge_markers_after_edge_types() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

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

    let (graph, view, editor_config) =
        insert_graph_view_editor_config_with(&mut host, graph_value, |state| {
            state.runtime_tuning.only_render_visible_elements = false;
            state.interaction.frame_view_duration_ms = 0;
        });
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let edge_types = NodeGraphEdgeTypes::new().with_fallback(|_g, _e, _style, mut h| {
        h.start_marker = Some(EdgeMarker::arrow(12.0));
        h.end_marker = Some(EdgeMarker::arrow(12.0));
        h
    });

    let mut canvas = new_canvas!(host, graph.clone(), view.clone(), editor_config.clone())
        .with_edge_types(edge_types);
    let mut tree = UiTree::<TestUiHostImpl>::default();

    let mut services = CaptureServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let len_base = capture_arrow_axis_lengths(&services);
    assert_eq!(
        len_base.len(),
        2,
        "expected start+end marker before skin override"
    );
    let avg = |v: &[f32]| -> f32 { v.iter().sum::<f32>() / v.len().max(1) as f32 };
    let base = avg(&len_base);

    let mut canvas = new_canvas!(host, graph, view, editor_config)
        .with_edge_types(
            NodeGraphEdgeTypes::new().with_fallback(|_g, _e, _style, mut h| {
                h.start_marker = Some(EdgeMarker::arrow(12.0));
                h.end_marker = Some(EdgeMarker::arrow(12.0));
                h
            }),
        )
        .with_skin(Arc::new(MarkerChromeSkin {
            start: EdgeMarker::arrow(24.0),
            end: EdgeMarker::arrow(24.0),
        }));

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let len_override = capture_arrow_axis_lengths(&services);
    assert_eq!(
        len_override.len(),
        2,
        "expected start+end marker after skin override"
    );

    let over = avg(&len_override);
    let ratio = if base > 1.0e-6 { over / base } else { 0.0 };
    let err = (ratio - 2.0).abs();
    assert!(
        err <= 1.0e-2,
        "expected skin marker size to override edgeTypes (24 vs 12 => ratio~2); base={base} over={over} ratio={ratio} err={err}"
    );
}
