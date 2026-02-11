use std::sync::Arc;

use fret_core::{
    Color, NodeId as UiNodeId, Point, Px, Rect, Scene, SceneOp, Size, TextBlobId, Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::NodeGraphCanvas;
use crate::ui::presenter::{EdgeRenderHint, EdgeRouteKind, NodeGraphPresenter};
use crate::ui::style::NodeGraphStyle;

use super::{TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_ports};

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

struct ColorOverridePresenter {
    label: Arc<str>,
    color: Color,
}

impl NodeGraphPresenter for ColorOverridePresenter {
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
            color: Some(self.color),
            route: EdgeRouteKind::Straight,
            width_mul: 1.0,
            ..EdgeRenderHint::default()
        }
    }
}

#[test]
fn edge_label_border_uses_edge_render_hint_color_override() {
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
    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
        s.interaction.frame_view_duration_ms = 0;
        s.interaction.bezier_hit_test_steps = 8;
    });

    let override_color = Color {
        r: 0.95,
        g: 0.35,
        b: 0.35,
        a: 0.9,
    };
    let presenter = ColorOverridePresenter {
        label: Arc::<str>::from("EdgeLabel"),
        color: override_color,
    };

    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(presenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CaptureServices::default();

    // If edge label caching is enabled, allow a couple frames for budgeted warmup.
    for _ in 0..8 {
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
        let tile_build_done = canvas.edge_labels_build_states.is_empty();
        let single_build_done = canvas.edge_labels_build_state.is_none();
        if tile_build_done && single_build_done {
            break;
        }
    }

    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let ops = scene.ops();

    let mut found = false;
    for ix in 0..ops.len().saturating_sub(1) {
        let SceneOp::Quad {
            order,
            background,
            border_paint,
            ..
        } = ops[ix]
        else {
            continue;
        };
        if order != fret_core::DrawOrder(2) {
            continue;
        }
        if background != fret_core::Paint::Solid(canvas.style.edge_label_background) {
            continue;
        }
        if border_paint != fret_core::Paint::Solid(override_color) {
            continue;
        }
        if matches!(ops[ix + 1], SceneOp::Text { .. }) {
            found = true;
            break;
        }
    }

    assert!(
        found,
        "expected edge label quad to use EdgeRenderHint.color as its border color override"
    );
}
