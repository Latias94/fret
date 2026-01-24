use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fret_core::{AppWindowId, NodeId as UiNodeId, Point, Px, Rect, Scene, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Graph, PortId};
use crate::io::NodeGraphViewState;
use crate::ui::NodeGraphCanvas;
use crate::ui::presenter::NodeGraphPresenter;

use super::{NullServices, TestUiHostImpl, make_test_graph_two_nodes_with_size};

struct CountingPresenter {
    node_title_calls: Arc<AtomicUsize>,
}

impl NodeGraphPresenter for CountingPresenter {
    fn node_title(&self, _graph: &Graph, node: crate::core::NodeId) -> Arc<str> {
        self.node_title_calls.fetch_add(1, Ordering::Relaxed);
        Arc::<str>::from(format!("node {:?}", node))
    }

    fn port_label(&self, _graph: &Graph, port: PortId) -> Arc<str> {
        Arc::<str>::from(format!("port {:?}", port))
    }
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut NullServices,
    bounds: Rect,
) {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

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
}

#[test]
fn only_render_visible_elements_controls_render_culling_work() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos.x = 50_000.0;
    graph_value.nodes.get_mut(&b).expect("node b exists").pos.y = 0.0;

    let counts = Arc::new(AtomicUsize::new(0));

    // With culling enabled, we expect far-away nodes to be skipped during render-data collection.
    let titles_culled = {
        let mut host = TestUiHostImpl::default();
        let graph = host.models.insert(graph_value.clone());
        let view = host.models.insert(NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = crate::core::CanvasPoint::default();
            s.zoom = 1.0;
            s.interaction.only_render_visible_elements = true;
            s.interaction.frame_view_duration_ms = 0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(CountingPresenter {
            node_title_calls: counts.clone(),
        });
        let mut tree = UiTree::<TestUiHostImpl>::default();
        let mut services = NullServices::default();
        paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

        counts.load(Ordering::Relaxed)
    };

    counts.store(0, Ordering::Relaxed);

    // With culling disabled, we expect both nodes to be visited even if offscreen.
    let titles_full = {
        let mut host = TestUiHostImpl::default();
        let graph = host.models.insert(graph_value);
        let view = host.models.insert(NodeGraphViewState::default());
        let _ = view.update(&mut host, |s, _cx| {
            s.pan = crate::core::CanvasPoint::default();
            s.zoom = 1.0;
            s.interaction.only_render_visible_elements = false;
            s.interaction.frame_view_duration_ms = 0;
        });

        let mut canvas = NodeGraphCanvas::new(graph, view).with_presenter(CountingPresenter {
            node_title_calls: counts.clone(),
        });
        let mut tree = UiTree::<TestUiHostImpl>::default();
        let mut services = NullServices::default();
        paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

        counts.load(Ordering::Relaxed)
    };

    assert!(titles_culled >= 1, "expected at least one node title call");
    assert!(
        titles_full > titles_culled,
        "expected disabling culling to increase work (culled={titles_culled}, full={titles_full})"
    );
    assert_ne!(a, b);
}
