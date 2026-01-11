use std::any::TypeId;
use std::sync::Arc;

use fret_core::{
    AppWindowId, NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle,
    Point, Px, Rect, Scene, Size, SvgId, TextBlobId, TextConstraints, TextMetrics, TextService,
    TextStyle, Transform2D,
};
use fret_runtime::ModelId;
use fret_ui::Invalidation;
use fret_ui::UiTree;
use fret_ui::retained_bridge::Widget as _;

use crate::core::{
    CanvasPoint, Edge, EdgeId, EdgeKind, Graph, GraphId, Node, NodeId, NodeKindKey, Port,
    PortCapacity, PortDirection, PortId, PortKey, PortKind,
};
use crate::ui::presenter::{EdgeMarker, EdgeRenderHint, NodeGraphPresenter};

use super::super::NodeGraphCanvas;
use super::TestUiHostImpl;

#[derive(Default)]
struct CountingServices {
    text_prepare: usize,
    text_release: usize,
    path_prepare: usize,
    path_release: usize,
}

impl TextService for CountingServices {
    fn prepare(
        &mut self,
        text: &str,
        _style: &TextStyle,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        self.text_prepare += 1;
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(text.len() as f32 * 7.0), Px(14.0)),
                baseline: Px(11.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {
        self.text_release += 1;
    }
}

impl fret_core::PathService for CountingServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        self.path_prepare += 1;
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {
        self.path_release += 1;
    }
}

impl fret_core::SvgService for CountingServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

#[derive(Default)]
struct CacheTestPresenter;

impl NodeGraphPresenter for CacheTestPresenter {
    fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
        Arc::<str>::from("Node")
    }

    fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
        Arc::<str>::from("Port")
    }

    fn edge_render_hint(
        &self,
        _graph: &Graph,
        _edge: EdgeId,
        _style: &crate::ui::style::NodeGraphStyle,
    ) -> EdgeRenderHint {
        EdgeRenderHint {
            label: Some(Arc::<str>::from("Edge")),
            width_mul: 1.0,
            start_marker: Some(EdgeMarker::arrow(10.0)),
            end_marker: Some(EdgeMarker::arrow(10.0)),
            ..EdgeRenderHint::default()
        }
    }
}

fn make_graph_two_nodes_one_edge() -> Graph {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let n1 = NodeId::new();
    let n2 = NodeId::new();
    let out = PortId::new();
    let inn = PortId::new();

    graph.nodes.insert(
        n1,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 0.0, y: 0.0 },
            selectable: None,
            draggable: None,
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        n2,
        Node {
            kind,
            kind_version: 1,
            pos: CanvasPoint { x: 260.0, y: 0.0 },
            selectable: None,
            draggable: None,
            parent: None,
            size: None,
            collapsed: false,
            ports: vec![inn],
            data: serde_json::Value::Null,
        },
    );

    graph.ports.insert(
        out,
        Port {
            node: n1,
            key: PortKey::new("out"),
            dir: PortDirection::Out,
            kind: PortKind::Data,
            capacity: PortCapacity::Multi,
            ty: None,
            data: serde_json::Value::Null,
        },
    );
    graph.ports.insert(
        inn,
        Port {
            node: n2,
            key: PortKey::new("in"),
            dir: PortDirection::In,
            kind: PortKind::Data,
            capacity: PortCapacity::Single,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    let edge = EdgeId::new();
    graph.edges.insert(
        edge,
        Edge {
            from: out,
            to: inn,
            kind: EdgeKind::Data,
            selectable: None,
        },
    );

    graph
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CountingServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    let mut observe_model = |_id: ModelId, _inv: Invalidation| {};
    let mut observe_global = |_id: TypeId, _inv: Invalidation| {};

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
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
        scene: &mut scene,
    };

    canvas.paint(&mut cx);
    scene
}

fn layout_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CountingServices,
    bounds: Rect,
) {
    let mut observe_model = |_id: ModelId, _inv: Invalidation| {};
    let mut observe_global = |_id: TypeId, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::LayoutCx {
        app: host,
        tree,
        node: UiNodeId::default(),
        window: Some(AppWindowId::default()),
        focus: None,
        children: &[],
        bounds,
        available: bounds.size,
        scale_factor: 1.0,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
    };

    let _ = canvas.layout(&mut cx);
}

#[test]
fn paint_reuses_cached_paths_and_text_between_frames() {
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(make_graph_two_nodes_one_edge());
    let view = host.models.insert(crate::io::NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(CacheTestPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let first = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert!(first.ops_len() > 0);
    let text_prepare_1 = services.text_prepare;
    let path_prepare_1 = services.path_prepare;
    assert!(text_prepare_1 > 0);
    assert!(path_prepare_1 > 0);

    let _ = view.update(&mut host, |s, _cx| {
        s.pan.x += 120.0;
    });

    let second = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert!(second.ops_len() > 0);
    assert_eq!(services.text_prepare, text_prepare_1);
    assert_eq!(services.path_prepare, path_prepare_1);
}

#[test]
fn auto_measure_dedupes_text_measure_for_repeated_labels() {
    let mut host = TestUiHostImpl::default();
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    for ix in 0..12 {
        let node = NodeId::new();
        let in_port = PortId::new();
        let out_port = PortId::new();
        graph.nodes.insert(
            node,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint {
                    x: ix as f32 * 260.0,
                    y: 0.0,
                },
                selectable: None,
                draggable: None,
                parent: None,
                size: None,
                collapsed: false,
                ports: vec![in_port, out_port],
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            in_port,
            Port {
                node,
                key: PortKey::new("in"),
                dir: PortDirection::In,
                kind: PortKind::Data,
                capacity: PortCapacity::Single,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
        graph.ports.insert(
            out_port,
            Port {
                node,
                key: PortKey::new("out"),
                dir: PortDirection::Out,
                kind: PortKind::Data,
                capacity: PortCapacity::Multi,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
    }

    let graph = host.models.insert(graph);
    let view = host.models.insert(crate::io::NodeGraphViewState::default());
    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(CacheTestPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();
    layout_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(services.text_prepare, 2);
    assert_eq!(services.text_release, 2);
}
