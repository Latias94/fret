use std::any::TypeId;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{
    AppWindowId, NodeId as UiNodeId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle,
    Point, Px, Rect, Scene, Size, SvgId, TextBlobId, TextConstraints, TextInput, TextMetrics,
    TextService, Transform2D,
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

use super::prelude::NodeGraphCanvas;
use super::{TestUiHostImpl, insert_graph_view, make_host_graph_view};

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
        input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        self.text_prepare += 1;
        let text = input.text();
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

#[derive(Clone, Default)]
struct PresenterCallCounts {
    node_title: Arc<AtomicUsize>,
    port_label: Arc<AtomicUsize>,
    edge_render_hint: Arc<AtomicUsize>,
    edge_color: Arc<AtomicUsize>,
}

#[derive(Clone, Default)]
struct CountingPresenter {
    counts: PresenterCallCounts,
}

impl NodeGraphPresenter for CountingPresenter {
    fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
        self.counts.node_title.fetch_add(1, Ordering::Relaxed);
        Arc::<str>::from("Node")
    }

    fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
        self.counts.port_label.fetch_add(1, Ordering::Relaxed);
        Arc::<str>::from("Port")
    }

    fn edge_color(
        &self,
        graph: &Graph,
        edge: EdgeId,
        style: &crate::ui::style::NodeGraphStyle,
    ) -> fret_core::Color {
        self.counts.edge_color.fetch_add(1, Ordering::Relaxed);
        let Some(e) = graph.edges.get(&edge) else {
            return style.node_border;
        };
        match e.kind {
            crate::core::EdgeKind::Data => style.wire_color_data,
            crate::core::EdgeKind::Exec => style.wire_color_exec,
        }
    }

    fn edge_render_hint(
        &self,
        _graph: &Graph,
        _edge: EdgeId,
        _style: &crate::ui::style::NodeGraphStyle,
    ) -> EdgeRenderHint {
        self.counts.edge_render_hint.fetch_add(1, Ordering::Relaxed);
        EdgeRenderHint {
            width_mul: 1.0,
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
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
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
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            deletable: None,
            reconnectable: None,
        },
    );

    graph
}

fn make_graph_two_nodes_many_edges(edge_count: usize) -> Graph {
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
            connectable: None,
            deletable: None,
            parent: None,
            extent: None,
            expand_parent: None,
            size: None,
            hidden: false,
            collapsed: false,
            ports: vec![out],
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        n2,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 360.0, y: 0.0 },
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
            connectable: None,
            connectable_start: None,
            connectable_end: None,
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
            capacity: PortCapacity::Multi,
            connectable: None,
            connectable_start: None,
            connectable_end: None,
            ty: None,
            data: serde_json::Value::Null,
        },
    );

    for _ in 0..edge_count {
        graph.edges.insert(
            EdgeId::new(),
            Edge {
                from: out,
                to: inn,
                kind: EdgeKind::Data,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
    }

    graph
}

fn make_graph_two_nodes_many_ports_and_edges(edge_count: usize) -> Graph {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let n1 = NodeId::new();
    let n2 = NodeId::new();

    let mut out_ports: Vec<PortId> = Vec::with_capacity(edge_count);
    let mut in_ports: Vec<PortId> = Vec::with_capacity(edge_count);

    for ix in 0..edge_count {
        let out = PortId::new();
        let inn = PortId::new();
        out_ports.push(out);
        in_ports.push(inn);

        graph.ports.insert(
            out,
            Port {
                node: n1,
                key: PortKey::new(format!("out{ix}")),
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
        graph.ports.insert(
            inn,
            Port {
                node: n2,
                key: PortKey::new(format!("in{ix}")),
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
    }

    graph.nodes.insert(
        n1,
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
            ports: out_ports.clone(),
            data: serde_json::Value::Null,
        },
    );
    graph.nodes.insert(
        n2,
        Node {
            kind: kind.clone(),
            kind_version: 1,
            pos: CanvasPoint { x: 420.0, y: 0.0 },
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
            ports: in_ports.clone(),
            data: serde_json::Value::Null,
        },
    );

    for ix in 0..edge_count {
        graph.edges.insert(
            EdgeId::new(),
            Edge {
                from: out_ports[ix],
                to: in_ports[ix],
                kind: EdgeKind::Data,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
    }

    graph
}

fn make_graph_chain_edges(edge_count: usize, spacing: f32) -> Graph {
    let mut graph = Graph::new(GraphId::new());
    let kind = NodeKindKey::new("test.node");

    let mut nodes: Vec<NodeId> = Vec::with_capacity(edge_count + 1);
    let mut in_ports: Vec<PortId> = Vec::with_capacity(edge_count + 1);
    let mut out_ports: Vec<PortId> = Vec::with_capacity(edge_count + 1);

    for ix in 0..=edge_count {
        let node = NodeId::new();
        let inn = PortId::new();
        let out = PortId::new();
        nodes.push(node);
        in_ports.push(inn);
        out_ports.push(out);

        graph.nodes.insert(
            node,
            Node {
                kind: kind.clone(),
                kind_version: 1,
                pos: CanvasPoint {
                    x: ix as f32 * spacing,
                    y: 0.0,
                },
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
                ports: vec![inn, out],
                data: serde_json::Value::Null,
            },
        );

        graph.ports.insert(
            inn,
            Port {
                node,
                key: PortKey::new(format!("in{ix}")),
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

        graph.ports.insert(
            out,
            Port {
                node,
                key: PortKey::new(format!("out{ix}")),
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
    }

    for ix in 0..edge_count {
        graph.edges.insert(
            EdgeId::new(),
            Edge {
                from: out_ports[ix],
                to: in_ports[ix + 1],
                kind: EdgeKind::Data,
                selectable: None,
                deletable: None,
                reconnectable: None,
            },
        );
    }

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
        children_render_transform: None,
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
        pass_kind: fret_ui::layout_pass::LayoutPassKind::Final,
        scale_factor: 1.0,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
    };

    let _ = canvas.layout(&mut cx);
}

#[test]
fn paint_reuses_cached_paths_and_text_between_frames() {
    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_one_edge());
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
fn paint_reuses_static_node_scene_cache_without_revisiting_presenter() {
    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_one_edge());

    let presenter = CountingPresenter::default();
    let counts = presenter.counts.clone();

    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let first_node_title = counts.node_title.load(Ordering::Relaxed);
    let first_port_label = counts.port_label.load(Ordering::Relaxed);
    assert!(first_node_title > 0);
    assert!(first_port_label > 0);

    let _ = view.update(&mut host, |s, _cx| {
        // Move within the static cache tile (should hit cache and avoid rebuilding node chrome).
        s.pan.x += 120.0;
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert_eq!(counts.node_title.load(Ordering::Relaxed), first_node_title);
    assert_eq!(counts.port_label.load(Ordering::Relaxed), first_port_label);
}

#[test]
fn paint_reuses_static_edge_scene_cache_without_revisiting_presenter() {
    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_one_edge());

    let presenter = CountingPresenter::default();
    let counts = presenter.counts.clone();
    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let first_edge_hint = counts.edge_render_hint.load(Ordering::Relaxed);
    let first_edge_color = counts.edge_color.load(Ordering::Relaxed);

    assert!(
        first_edge_hint > 0,
        "edge_render_hint was not called; edge_color calls: {first_edge_color}"
    );
    assert!(first_edge_color > 0);

    let _ = view.update(&mut host, |s, _cx| {
        s.pan.x += 120.0;
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert_eq!(
        counts.edge_render_hint.load(Ordering::Relaxed),
        first_edge_hint
    );
    assert_eq!(counts.edge_color.load(Ordering::Relaxed), first_edge_color);
}

#[test]
fn paint_invalidates_static_edge_scene_cache_when_edge_types_change() {
    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_one_edge());

    let presenter = CountingPresenter::default();
    let counts = presenter.counts.clone();
    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let first_edge_hint = counts.edge_render_hint.load(Ordering::Relaxed);
    assert!(first_edge_hint > 0);

    // Mutate the edge type registry so `edge_types_rev` changes and static scene cache keys miss.
    canvas.edge_types = Some(crate::ui::NodeGraphEdgeTypes::new().with_fallback_path(
        |_g, _e, _style, _hint, input| {
            Some(crate::ui::edge_types::EdgeCustomPath {
                cache_key: 1,
                commands: vec![fret_core::PathCommand::MoveTo(input.from)],
            })
        },
    ));

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let second_edge_hint = counts.edge_render_hint.load(Ordering::Relaxed);
    assert!(
        second_edge_hint > first_edge_hint,
        "expected edge_render_hint to be revisited after edge_types_rev changes; before={first_edge_hint}, after={second_edge_hint}"
    );
}

#[test]
fn paint_reuses_static_edge_scene_cache_when_panning_back_across_tiles() {
    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_one_edge());

    let presenter = CountingPresenter::default();
    let mut canvas = NodeGraphCanvas::new(graph, view.clone()).with_presenter(presenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let entries_1 = canvas.edges_scene_cache.entries_len();
    let stats_1 = canvas.edges_scene_cache.stats();
    assert!(entries_1 > 0);

    let _ = view.update(&mut host, |s, _cx| {
        // Cross the static cache tile boundary while keeping the content in view.
        s.pan.x += 500.0;
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let entries_2 = canvas.edges_scene_cache.entries_len();
    let stats_2 = canvas.edges_scene_cache.stats();
    assert!(entries_2 > entries_1);
    assert!(stats_2.misses > stats_1.misses);

    let _ = view.update(&mut host, |s, _cx| {
        s.pan.x -= 500.0;
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let entries_3 = canvas.edges_scene_cache.entries_len();
    let stats_3 = canvas.edges_scene_cache.stats();
    assert_eq!(entries_3, entries_2);
    assert!(stats_3.hits > stats_2.hits);
}

#[test]
fn paint_warms_edge_label_scene_cache_incrementally() {
    #[derive(Default)]
    struct UniqueEdgeLabelPresenter;

    impl NodeGraphPresenter for UniqueEdgeLabelPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("Node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("Port")
        }

        fn edge_render_hint(
            &self,
            _graph: &Graph,
            edge: EdgeId,
            _style: &crate::ui::style::NodeGraphStyle,
        ) -> EdgeRenderHint {
            EdgeRenderHint {
                label: Some(Arc::<str>::from(format!("Edge {edge:?}"))),
                width_mul: 1.0,
                ..EdgeRenderHint::default()
            }
        }
    }

    let (mut host, graph, view) = make_host_graph_view(make_graph_two_nodes_many_edges(80));
    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(UniqueEdgeLabelPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_1 = services.text_prepare;
    assert!(
        prepare_1 < 80,
        "expected edge label warmup to be incremental; got {prepare_1} text prepares"
    );

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_2 = services.text_prepare;
    assert!(prepare_2 > prepare_1);
    assert!(
        prepare_2 - prepare_1 <= 64,
        "expected a bounded per-frame label budget; delta was {}",
        prepare_2 - prepare_1
    );

    // Run a few more frames; the warmup should eventually stabilize.
    for _ in 0..10 {
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    }
    let prepare_done = services.text_prepare;
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert_eq!(services.text_prepare, prepare_done);
}

#[test]
fn paint_warms_edge_scene_cache_incrementally() {
    #[derive(Default)]
    struct MarkerPresenter;

    impl NodeGraphPresenter for MarkerPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("Node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("Port")
        }

        fn edge_render_hint(
            &self,
            _graph: &Graph,
            edge: EdgeId,
            _style: &crate::ui::style::NodeGraphStyle,
        ) -> EdgeRenderHint {
            EdgeRenderHint {
                label: Some(Arc::<str>::from(format!("Edge {edge:?}"))),
                width_mul: 1.0,
                start_marker: Some(EdgeMarker::arrow(10.0)),
                end_marker: Some(EdgeMarker::arrow(10.0)),
                ..EdgeRenderHint::default()
            }
        }
    }

    let (mut host, graph, view) =
        make_host_graph_view(make_graph_two_nodes_many_ports_and_edges(240));
    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(MarkerPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_1 = services.path_prepare;
    assert!(
        prepare_1 > 0,
        "expected at least some path work in the first frame"
    );

    // Marker budget should prevent building the entire tile in one frame.
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_2 = services.path_prepare;
    assert!(prepare_2 > prepare_1);

    // Eventually stabilizes once the cache is fully populated.
    for _ in 0..16 {
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    }
    let prepare_done = services.path_prepare;
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert_eq!(services.path_prepare, prepare_done);
}

#[test]
fn paint_warms_edge_label_scene_cache_incrementally_for_large_viewport_tiles() {
    #[derive(Default)]
    struct UniqueEdgeLabelPresenter;

    impl NodeGraphPresenter for UniqueEdgeLabelPresenter {
        fn node_title(&self, _graph: &Graph, _node: NodeId) -> Arc<str> {
            Arc::<str>::from("Node")
        }

        fn port_label(&self, _graph: &Graph, _port: PortId) -> Arc<str> {
            Arc::<str>::from("Port")
        }

        fn edge_render_hint(
            &self,
            _graph: &Graph,
            edge: EdgeId,
            _style: &crate::ui::style::NodeGraphStyle,
        ) -> EdgeRenderHint {
            EdgeRenderHint {
                label: Some(Arc::<str>::from(format!("Edge {edge:?}"))),
                width_mul: 1.0,
                ..EdgeRenderHint::default()
            }
        }
    }

    let (mut host, graph, view) = make_host_graph_view(make_graph_chain_edges(150, 40.0));
    let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(UniqueEdgeLabelPresenter);

    // Large enough to force multi-tile static edge cache path.
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(5000.0), Px(700.0)),
    );

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_1 = services.text_prepare;

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let prepare_2 = services.text_prepare;
    assert!(
        prepare_2 > prepare_1,
        "expected edge label warmup to be incremental under a per-frame budget"
    );
    assert!(
        prepare_2 - prepare_1 <= 128,
        "expected a bounded per-frame label budget; delta was {}",
        prepare_2 - prepare_1
    );

    for _ in 0..32 {
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    }
    let prepare_done = services.text_prepare;
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    assert_eq!(services.text_prepare, prepare_done);
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
                connectable: None,
                deletable: None,
                parent: None,
                extent: None,
                expand_parent: None,
                size: None,
                hidden: false,
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
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
                connectable: None,
                connectable_start: None,
                connectable_end: None,
                ty: None,
                data: serde_json::Value::Null,
            },
        );
    }

    let (graph, view) = insert_graph_view(&mut host, graph);
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
