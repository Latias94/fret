use fret_core::{AppWindowId, NodeId as UiNodeId, Point, Px, Rect, Scene, Size, Transform2D};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::{NodeGraphCanvas, NodeGraphColorMode, NodeGraphStyle};

use super::{NullServices, TestUiHostImpl, make_host_graph_view, make_test_graph_two_nodes};

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

    let mut cx = fret_ui::retained_bridge::PaintCx::new(
        host,
        tree,
        UiNodeId::default(),
        Some(AppWindowId::default()),
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

    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let mut canvas = new_canvas!(host, graph, view).with_color_mode(NodeGraphColorMode::System);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let theme = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(
        canvas.style.paint.background,
        theme.color_token("background")
    );
}

#[test]
fn color_mode_light_forces_light_palette() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let mut canvas = new_canvas!(host, graph, view).with_color_mode(NodeGraphColorMode::Light);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(
        canvas.style.paint.background,
        NodeGraphStyle::light_defaults().paint.background
    );
}

#[test]
fn color_mode_dark_forces_dark_palette() {
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    );

    let (graph_value, _a, _b) = make_test_graph_two_nodes();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let mut canvas = new_canvas!(host, graph, view).with_color_mode(NodeGraphColorMode::Dark);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds);

    assert_eq!(
        canvas.style.paint.background,
        NodeGraphStyle::default().paint.background
    );
}
