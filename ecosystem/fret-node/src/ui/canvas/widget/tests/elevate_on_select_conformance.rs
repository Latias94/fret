use std::sync::Arc;

use fret_core::{Color, Point, Px, Rect, Scene, SceneOp, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::io::NodeGraphViewState;
use crate::ui::presenter::NodeGraphPresenter;
use crate::ui::{NodeGraphCanvas, NodeGraphStyle};

use super::{NullServices, TestUiHostImpl, make_test_graph_two_nodes_with_size};

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

    let mut cx = fret_ui::retained_bridge::PaintCx {
        app: host,
        tree,
        node: fret_core::NodeId::default(),
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

struct TestPresenter;

impl NodeGraphPresenter for TestPresenter {
    fn node_title(&self, _graph: &Graph, node: NodeId) -> Arc<str> {
        Arc::<str>::from(format!("node {node:?}"))
    }

    fn port_label(&self, _graph: &Graph, port: PortId) -> Arc<str> {
        Arc::<str>::from(format!("port {port:?}"))
    }
}

#[test]
fn elevate_nodes_on_select_draws_selected_node_body_last() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, a, b) = make_test_graph_two_nodes_with_size();

    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.pan = CanvasPoint::default();
        s.zoom = 1.0;
        s.draw_order = vec![a, b];
        s.selected_nodes = vec![a];
        s.interaction.only_render_visible_elements = false;
        s.interaction.elevate_nodes_on_select = true;
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(TestPresenter);
    let snapshot = canvas.sync_view_state(&mut host);
    let (geom, _index) = canvas.canvas_derived(&host, &snapshot);
    let rect_a = geom.nodes.get(&a).expect("node a exists").rect;
    let rect_b = geom.nodes.get(&b).expect("node b exists").rect;

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let mut last_a = None;
    let mut last_b = None;
    for (ix, op) in scene.ops().iter().enumerate() {
        let SceneOp::Quad { rect, order, .. } = op else {
            continue;
        };
        if *order != fret_core::DrawOrder(3) {
            continue;
        }
        if *rect == rect_a {
            last_a = Some(ix);
        }
        if *rect == rect_b {
            last_b = Some(ix);
        }
    }

    let last_a = last_a.expect("expected a quad for node a");
    let last_b = last_b.expect("expected a quad for node b");
    assert!(
        last_a > last_b,
        "expected selected node body to be repainted on top (a={last_a}, b={last_b})"
    );
}

struct EdgeColorPresenter {
    red: EdgeId,
    green: EdgeId,
}

impl NodeGraphPresenter for EdgeColorPresenter {
    fn node_title(&self, _graph: &Graph, node: NodeId) -> Arc<str> {
        Arc::<str>::from(format!("node {node:?}"))
    }

    fn port_label(&self, _graph: &Graph, port: PortId) -> Arc<str> {
        Arc::<str>::from(format!("port {port:?}"))
    }

    fn edge_color(&self, _graph: &Graph, edge: EdgeId, _style: &NodeGraphStyle) -> Color {
        if edge == self.red {
            Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }
        } else if edge == self.green {
            Color {
                r: 0.0,
                g: 1.0,
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

fn edge_paths_in_scene(scene: &Scene) -> Vec<Color> {
    let mut out = Vec::new();
    for op in scene.ops().iter() {
        let SceneOp::Path { order, color, .. } = op else {
            continue;
        };
        if *order != fret_core::DrawOrder(2) {
            continue;
        }
        out.push(*color);
    }
    out
}

#[test]
fn elevate_edges_on_select_controls_selection_z_order() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut graph_value = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let a = NodeId::new();
    let b = NodeId::new();
    let c = NodeId::new();

    let a_out = PortId::new();
    let b_in = PortId::new();
    let c_out = PortId::new();

    graph_value.nodes.insert(
        a,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
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
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 260.0, y: 0.0 },
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
    graph_value.nodes.insert(
        c,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 520.0, y: 0.0 },
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
            ports: vec![c_out],
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
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    graph_value.ports.insert(
        c_out,
        Port {
            node: c,
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

    let edge_low = EdgeId::new();
    let edge_high = EdgeId::new();
    graph_value.edges.insert(
        edge_low,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );
    graph_value.edges.insert(
        edge_high,
        Edge {
            kind: EdgeKind::Data,
            from: c_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let paths_with = |elevate: bool| {
        let mut host = TestUiHostImpl::default();
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = CanvasPoint::default();
            s.zoom = 1.0;
            s.draw_order = vec![a, b, c];
            s.selected_edges = vec![edge_low];
            s.interaction.only_render_visible_elements = false;
            s.interaction.elevate_edges_on_select = elevate;
            s.interaction.edges_reconnectable = false;
            s.interaction.frame_view_duration_ms = 0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(EdgeColorPresenter {
            red: edge_low,
            green: edge_high,
        });
        let mut tree = UiTree::<TestUiHostImpl>::default();
        let mut services = NullServices::default();
        let scene = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
        edge_paths_in_scene(&scene)
    };

    let elevated = paths_with(true);
    let stable = paths_with(false);

    assert!(
        elevated.len() >= 2,
        "expected at least two edge paths, got {}",
        elevated.len()
    );
    assert!(
        stable.len() >= 2,
        "expected at least two edge paths, got {}",
        stable.len()
    );

    // Selection elevation should cause the selected edge (red) to be drawn after the higher-rank
    // edge (green). We key on color ordering to avoid exposing internal IDs.
    let red = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    let green = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    let last_red_elevated = elevated
        .iter()
        .rposition(|c| *c == red)
        .expect("expected a red edge path");
    let last_green_elevated = elevated
        .iter()
        .rposition(|c| *c == green)
        .expect("expected a green edge path");
    assert!(
        last_red_elevated > last_green_elevated,
        "expected selected edge to be drawn last when elevation is enabled"
    );

    let last_red_stable = stable
        .iter()
        .rposition(|c| *c == red)
        .expect("expected a red edge path");
    let last_green_stable = stable
        .iter()
        .rposition(|c| *c == green)
        .expect("expected a green edge path");
    assert!(
        last_red_stable < last_green_stable,
        "expected stable rank ordering when elevation is disabled"
    );
}
