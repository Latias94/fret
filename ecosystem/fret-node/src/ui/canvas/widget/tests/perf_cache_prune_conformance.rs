use std::any::TypeId;
use std::sync::Arc;

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
            start_marker: Some(EdgeMarker::arrow(10.0)),
            end_marker: Some(EdgeMarker::arrow(10.0)),
            ..EdgeRenderHint::default()
        }
    }
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
        window: Some(AppWindowId::default()),
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

#[test]
fn paint_cache_prune_releases_old_path_and_text_entries() {
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(make_graph_chain_edges(220, 80.0));
    let view = host.models.insert(crate::io::NodeGraphViewState::default());

    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.paint_cache_prune.max_age_frames = 2;
        // Use a large entry budget so this test is primarily driven by max_age eviction.
        s.interaction.paint_cache_prune.max_entries = 10_000;
    });

    let mut canvas =
        NodeGraphCanvas::new(graph, view.clone()).with_presenter(UniqueEdgeLabelPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let release_0 = (services.text_release, services.path_release);
    assert!(services.text_prepare > 0);
    assert!(services.path_prepare > 0);

    // Walk through multiple distinct viewports so previously cached resources age out and are
    // released by prune.
    for step in 0..10 {
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = CanvasPoint {
                x: 2400.0 * (step as f32 + 1.0),
                y: 0.0,
            };
        });
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    }

    assert!(
        services.text_release > release_0.0 || services.path_release > release_0.1,
        "expected pruning to release at least some cached resources; text_release {}->{} path_release {}->{}",
        release_0.0,
        services.text_release,
        release_0.1,
        services.path_release
    );
}

#[test]
fn static_scene_tile_cache_does_not_grow_unbounded_while_panning_across_many_tiles() {
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(make_graph_chain_edges(80, 260.0));
    let view = host.models.insert(crate::io::NodeGraphViewState::default());
    let mut canvas =
        NodeGraphCanvas::new(graph, view.clone()).with_presenter(UniqueEdgeLabelPresenter);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );
    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    for step in 0..64 {
        let _ = view.update(&mut host, |s, _cx| {
            s.pan.x = 2500.0 * step as f32;
        });
        let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
        assert!(
            canvas.edges_scene_cache.entries_len()
                <= NodeGraphCanvas::STATIC_SCENE_TILE_CACHE_MAX_ENTRIES,
            "expected static tile cache entry count to remain bounded"
        );
    }
}
