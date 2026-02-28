use std::sync::Arc;

use fret_core::{Color, DrawOrder, Paint, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{
    Edge, EdgeId, EdgeKind, Graph, GraphId, NodeId, NodeKindKey, Port, PortCapacity, PortDirection,
    PortId, PortKey, PortKind,
};
use crate::ui::presenter::NodeGraphPresenter;
use crate::ui::{
    InteractionChromeHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle, WireOutlineHint,
};

use super::{NullServices, TestUiHostImpl, insert_view};

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
    fn node_title(&self, _graph: &Graph, node: NodeId) -> Arc<str> {
        Arc::<str>::from(format!("node {node:?}"))
    }

    fn port_label(&self, _graph: &Graph, port: PortId) -> Arc<str> {
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

struct WireOutlineSkin {
    outline: WireOutlineHint,
}

impl NodeGraphSkin for WireOutlineSkin {
    fn interaction_chrome_hint(
        &self,
        _graph: &Graph,
        _style: &NodeGraphStyle,
    ) -> InteractionChromeHint {
        InteractionChromeHint {
            wire_outline_selected: Some(self.outline),
            wire_glow_selected: None,
            ..InteractionChromeHint::default()
        }
    }
}

fn edge_path_colors(scene: &Scene) -> Vec<Color> {
    let mut out = Vec::new();
    for op in scene.ops().iter() {
        let SceneOp::Path { order, paint, .. } = op else {
            continue;
        };
        if *order != DrawOrder(2) {
            continue;
        }
        if let Paint::Solid(color) = *paint {
            out.push(color);
        }
    }
    out
}

#[test]
fn skin_wire_outline_selected_draws_outline_path_before_core() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    let a_out = PortId::new();
    let b_in = PortId::new();

    graph_value.nodes.insert(
        a,
        crate::core::Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![a_out],
            data: serde_json::Value::Null,
        },
    );
    graph_value.nodes.insert(
        b,
        crate::core::Node {
            kind,
            kind_version: 1,
            pos: crate::core::CanvasPoint { x: 420.0, y: 0.0 },
            selectable: None,
            draggable: None,
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![b_in],
            data: serde_json::Value::Null,
        },
    );

    graph_value.ports.insert(
        a_out,
        Port {
            node: a,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        b_in,
        Port {
            node: b,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

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
    let _ = view.update(&mut host, |s, _cx| {
        s.pan = crate::core::CanvasPoint::default();
        s.zoom = 1.0;
        s.selected_edges = vec![edge_id];
        s.interaction.only_render_visible_elements = false;
        s.interaction.frame_view_duration_ms = 0;
    });

    let outline_color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.3,
    };
    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_style(style)
        .with_presenter(EdgeColorPresenter { edge: edge_id })
        .with_skin(Arc::new(WireOutlineSkin {
            outline: WireOutlineHint {
                width_mul: 1.8,
                color: outline_color,
            },
        }));

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let colors = edge_path_colors(&scene);
    assert!(
        colors.len() >= 2,
        "expected at least two edge Path ops (outline + core)"
    );

    let outline_hits = colors.iter().filter(|c| **c == outline_color).count();
    assert_eq!(outline_hits, 1, "expected exactly one outline Path op");

    let core_color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let core_hits = colors.iter().filter(|c| **c == core_color).count();
    assert_eq!(core_hits, 1, "expected exactly one core Path op");

    assert_eq!(
        colors[0], outline_color,
        "expected outline to be drawn before core stroke"
    );
    assert_eq!(colors[1], core_color, "expected core stroke after outline");
}
