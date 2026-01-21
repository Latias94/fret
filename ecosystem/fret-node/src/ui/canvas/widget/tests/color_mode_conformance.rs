use fret_core::{AppWindowId, NodeId as UiNodeId, Point, Px, Rect, Scene, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::io::NodeGraphViewState;
use crate::ui::{NodeGraphCanvas, NodeGraphColorMode, NodeGraphStyle};

use super::{NullServices, TestUiHostImpl, make_test_graph_two_nodes};

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut NullServices,
    bounds: Rect,
) -> fret_ui::ThemeSnapshot {
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

    let snapshot = cx.theme().snapshot();
    canvas.paint(&mut cx);
    snapshot
}

#[test]
fn color_mode_system_syncs_style_from_theme() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_color_mode(NodeGraphColorMode::System);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let theme = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(canvas.style.background, theme.color_required("background"));
}

#[test]
fn color_mode_light_forces_light_palette() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_color_mode(NodeGraphColorMode::Light);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(
        canvas.style.background,
        NodeGraphStyle::xyflow_light_defaults().background
    );
}

#[test]
fn color_mode_dark_forces_dark_palette() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let mut host = TestUiHostImpl::default();
    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let graph = host.models.insert(graph_value);
    let view = host.models.insert(NodeGraphViewState::default());

    let mut canvas = NodeGraphCanvas::new(graph, view).with_color_mode(NodeGraphColorMode::Dark);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(
        canvas.style.background,
        NodeGraphStyle::default().background
    );
}
