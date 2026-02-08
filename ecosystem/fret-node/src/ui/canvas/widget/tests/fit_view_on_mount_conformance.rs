use fret_core::{AppWindowId, NodeId as UiNodeId, Point, Px, Rect, Size};
use fret_ui::UiTree;
use fret_ui::layout_pass::LayoutPassKind;
use fret_ui::retained_bridge::Widget as _;

use crate::core::CanvasPoint;

use super::prelude::NodeGraphCanvas;
use super::{NullServices, TestUiHostImpl, insert_view, make_test_graph_two_nodes_with_size};

fn layout_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut NullServices,
    bounds: Rect,
) {
    let mut observe_model = |_id, _inv| {};
    let mut observe_global = |_id, _inv| {};

    let mut cx = fret_ui::retained_bridge::LayoutCx {
        app: host,
        tree,
        node: UiNodeId::default(),
        window: Some(AppWindowId::default()),
        focus: None,
        children: &[],
        bounds,
        available: bounds.size,
        pass_kind: LayoutPassKind::Final,
        scale_factor: 1.0,
        services,
        observe_model: &mut observe_model,
        observe_global: &mut observe_global,
    };

    let _ = canvas.layout(&mut cx);
}

#[test]
fn fit_view_on_mount_frames_all_nodes_once() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (mut graph_value, a, b) = make_test_graph_two_nodes_with_size();
    graph_value.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 5000.0, y: 0.0 };

    // Expected viewport by direct framing (immediate).
    let mut host_expected = TestUiHostImpl::default();
    let graph_expected = host_expected.models.insert(graph_value.clone());
    let view_expected = insert_view(&mut host_expected);
    let _ = view_expected.update(&mut host_expected, |s, _cx| {
        s.interaction.frame_view_duration_ms = 0;
    });
    let mut canvas_expected = NodeGraphCanvas::new(graph_expected, view_expected);
    assert!(canvas_expected.frame_nodes_in_view(&mut host_expected, None, bounds, &[a, b]));
    let expected = canvas_expected.sync_view_state(&mut host_expected);

    // Actual via one-shot fit-view on mount (layout).
    let mut host = TestUiHostImpl::default();
    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let _ = view.update(&mut host, |s, _cx| {
        s.interaction.frame_view_duration_ms = 0;
    });

    let mut canvas = NodeGraphCanvas::new(graph.clone(), view).with_fit_view_on_mount();

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    layout_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    let actual = canvas.sync_view_state(&mut host);
    assert!((actual.zoom - expected.zoom).abs() <= 1.0e-3);
    assert!((actual.pan.x - expected.pan.x).abs() <= 1.0e-2);
    assert!((actual.pan.y - expected.pan.y).abs() <= 1.0e-2);

    // Subsequent layouts should not re-fit even if nodes move.
    let _ = graph.update(&mut host, |g, _cx| {
        g.nodes.get_mut(&b).expect("node b exists").pos = CanvasPoint { x: 15000.0, y: 0.0 };
    });
    layout_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);
    let after = canvas.sync_view_state(&mut host);

    assert!((after.zoom - actual.zoom).abs() <= 1.0e-6);
    assert!((after.pan.x - actual.pan.x).abs() <= 1.0e-6);
    assert!((after.pan.y - actual.pan.y).abs() <= 1.0e-6);
}
