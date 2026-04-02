use std::sync::Arc;

use fret_core::{Color, DrawOrder, Paint, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind, Graph};
use crate::ui::presenter::NodeGraphPresenter;
use crate::ui::{
    InteractionChromeHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle, WireHighlightHint,
};

use super::{
    insert_editor_config_with, insert_view, make_test_graph_two_nodes_with_ports, NullServices,
    TestUiHostImpl,
};

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut NullServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx::new(
        host,
        tree,
        fret_core::NodeId::default(),
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
    scene
}

struct EdgeColorPresenter {
    edge: EdgeId,
}

impl NodeGraphPresenter for EdgeColorPresenter {
    fn node_title(&self, _graph: &Graph, node: crate::core::NodeId) -> Arc<str> {
        Arc::<str>::from(format!("node {node:?}"))
    }

    fn port_label(&self, _graph: &Graph, port: crate::core::PortId) -> Arc<str> {
        Arc::<str>::from(format!("port {port:?}"))
    }

    fn edge_color(&self, _graph: &Graph, edge: EdgeId, _style: &NodeGraphStyle) -> Color {
        if edge == self.edge {
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else {
            Color {
                r: 0.2,
                g: 0.2,
                b: 0.2,
                a: 1.0,
            }
        }
    }
}

struct WireHighlightSkin {
    selected: Option<WireHighlightHint>,
    hovered: Option<WireHighlightHint>,
}

impl NodeGraphSkin for WireHighlightSkin {
    fn interaction_chrome_hint(
        &self,
        _graph: &Graph,
        _style: &NodeGraphStyle,
    ) -> InteractionChromeHint {
        InteractionChromeHint {
            wire_highlight_selected: self.selected,
            wire_highlight_hovered: self.hovered,
            ..InteractionChromeHint::default()
        }
    }
}

fn edge_path_solid_colors(scene: &Scene) -> Vec<Color> {
    let mut out = Vec::new();
    for op in scene.ops().iter() {
        let SceneOp::Path { order, paint, .. } = op else {
            continue;
        };
        if *order != DrawOrder(2) {
            continue;
        }
        if let Paint::Solid(color) = paint.paint {
            out.push(color);
        }
    }
    out
}

#[test]
fn skin_wire_highlight_selected_draws_highlight_after_core() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

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

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = crate::core::CanvasPoint::default();
        s.zoom = 1.0;
        s.selected_edges = vec![edge_id];
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_editor_config_model(editor_config)
        .with_style(style)
        .with_presenter(EdgeColorPresenter { edge: edge_id })
        .with_skin(Arc::new(WireHighlightSkin {
            selected: Some(WireHighlightHint {
                width_mul: 0.7,
                alpha_mul: 0.5,
                color: Some(Color {
                    r: 0.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                }),
            }),
            hovered: None,
        }));

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let colors = edge_path_solid_colors(&scene);
    let core = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let highlight = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 0.5,
    };

    let core_hits = colors.iter().filter(|c| **c == core).count();
    assert_eq!(core_hits, 1, "expected exactly one core wire Path op");
    let highlight_hits = colors.iter().filter(|c| **c == highlight).count();
    assert_eq!(
        highlight_hits, 1,
        "expected exactly one highlight wire Path op"
    );

    let core_pos = colors
        .iter()
        .position(|c| *c == core)
        .expect("expected core wire Path op");
    let highlight_pos = colors
        .iter()
        .position(|c| *c == highlight)
        .expect("expected highlight wire Path op");
    assert!(
        core_pos < highlight_pos,
        "expected highlight to be drawn after the core stroke"
    );
}

#[test]
fn skin_wire_highlight_hovered_draws_highlight_after_core() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

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

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = crate::core::CanvasPoint::default();
        s.zoom = 1.0;
        s.selected_edges = Vec::new();
    });

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_editor_config_model(editor_config)
        .with_style(style)
        .with_presenter(EdgeColorPresenter { edge: edge_id })
        .with_skin(Arc::new(WireHighlightSkin {
            selected: None,
            hovered: Some(WireHighlightHint {
                width_mul: 0.75,
                alpha_mul: 0.5,
                color: Some(Color {
                    r: 0.0,
                    g: 0.0,
                    b: 1.0,
                    a: 1.0,
                }),
            }),
        }));
    canvas.interaction.hover_edge = Some(edge_id);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let colors = edge_path_solid_colors(&scene);
    let core = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let highlight = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 0.5,
    };

    let core_pos = colors
        .iter()
        .position(|c| *c == core)
        .expect("expected core wire Path op");
    let highlight_pos = colors
        .iter()
        .position(|c| *c == highlight)
        .expect("expected highlight wire Path op");
    assert!(
        core_pos < highlight_pos,
        "expected highlight to be drawn after the core stroke"
    );
}
