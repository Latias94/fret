use std::sync::Arc;

use fret_core::{
    AppWindowId, Color, NodeId as UiNodeId, Point, Px, Rect, Scene, Size, Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::ui::style::{NodeGraphBackgroundPattern, NodeGraphBackgroundStyle};
use crate::ui::{NodeGraphCanvas, NodeGraphColorMode, NodeGraphStyle};

use super::{
    NullServices, TestUiHostImpl, make_host_graph_view, make_test_graph_two_nodes_with_ports,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
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
fn background_style_updates_do_not_rebuild_canvas_derived_geometry() {
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let style = NodeGraphStyle::default();
    let mut canvas = NodeGraphCanvas::new(graph, view).with_style(style);

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);

    let background = NodeGraphBackgroundStyle {
        background: Color {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        },
        grid_pattern: NodeGraphBackgroundPattern::Dots,
        grid_spacing: 48.0,
        grid_minor_color: Color {
            r: 0.20,
            g: 0.20,
            b: 0.20,
            a: 1.0,
        },
        grid_major_every: 5,
        grid_major_color: Color {
            r: 0.35,
            g: 0.35,
            b: 0.35,
            a: 1.0,
        },
        grid_line_width: 2.0,
        grid_dot_size: 2.0,
        grid_cross_size: 6.0,
    };

    assert_ne!(canvas.background_style(), background);
    canvas = canvas.with_background_style(background);
    assert_eq!(canvas.background_style(), background);

    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);

    assert!(
        Arc::ptr_eq(&geom1, &geom2),
        "expected background style updates to preserve cached CanvasGeometry"
    );
    assert!(
        Arc::ptr_eq(&index1, &index2),
        "expected background style updates to preserve cached CanvasSpatialIndex"
    );
}

#[test]
fn background_style_override_survives_color_mode_theme_sync() {
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();
    let (mut host, graph, view) = make_host_graph_view(graph_value);

    let background = NodeGraphBackgroundStyle {
        background: Color {
            r: 0.05,
            g: 0.05,
            b: 0.05,
            a: 1.0,
        },
        grid_pattern: NodeGraphBackgroundPattern::Dots,
        grid_spacing: 48.0,
        grid_minor_color: Color {
            r: 0.20,
            g: 0.20,
            b: 0.20,
            a: 1.0,
        },
        grid_major_every: 5,
        grid_major_color: Color {
            r: 0.35,
            g: 0.35,
            b: 0.35,
            a: 1.0,
        },
        grid_line_width: 2.0,
        grid_dot_size: 2.0,
        grid_cross_size: 6.0,
    };

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_color_mode(NodeGraphColorMode::System)
        .with_background_style(background);

    let mut tree: UiTree<TestUiHostImpl> = UiTree::default();
    let mut services = NullServices::default();
    paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    assert_eq!(
        canvas.background_style(),
        background,
        "expected explicit background overrides to survive the canvas theme sync path"
    );
}
