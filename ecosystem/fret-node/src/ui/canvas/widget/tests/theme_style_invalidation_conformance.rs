use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Point, Px, Rect, Scene, Size, Transform2D};
use fret_runtime::ui_host::GlobalsHost;
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, Theme, ThemeConfig, UiTree};

use crate::ui::{NodeGraphCanvas, NodeGraphColorMode};

use super::{
    NullServices, TestUiHostImpl, insert_editor_config_with, insert_view,
    make_test_graph_two_nodes_with_ports,
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

#[test]
fn theme_palette_updates_do_not_rebuild_canvas_geometry_in_system_mode() {
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();

    let mut host = TestUiHostImpl::default();
    host.set_global(Theme::global(&host).clone());

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_color_mode(NodeGraphColorMode::System)
        .with_editor_config_model(editor_config);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    Theme::with_global_mut(&mut host, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.name =
            "theme_palette_updates_do_not_rebuild_canvas_geometry_in_system_mode".to_string();
        cfg.colors = HashMap::from([("background".to_string(), "#00ff00".to_string())]);
        theme.apply_config(&cfg);
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());
    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(Arc::ptr_eq(&geom1, &geom2));
    assert!(Arc::ptr_eq(&index1, &index2));
    assert_eq!(
        counters2.geom_rebuilds, counters1.geom_rebuilds,
        "palette-only theme changes must not rebuild derived geometry"
    );
    assert_eq!(
        counters2.index_rebuilds, counters1.index_rebuilds,
        "palette-only theme changes must not rebuild the spatial index"
    );
}

#[test]
fn theme_metric_updates_rebuild_canvas_geometry_in_system_mode() {
    let (graph_value, _a, _a_in, _a_out, _b, _b_in) = make_test_graph_two_nodes_with_ports();

    let mut host = TestUiHostImpl::default();
    host.set_global(Theme::global(&host).clone());

    let graph = host.models.insert(graph_value);
    let view = insert_view(&mut host);
    let editor_config = insert_editor_config_with(&mut host, |state| {
        state.runtime_tuning.only_render_visible_elements = false;
        state.interaction.frame_view_duration_ms = 0;
    });

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let mut canvas = NodeGraphCanvas::new(graph, view)
        .with_color_mode(NodeGraphColorMode::System)
        .with_editor_config_model(editor_config);

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = NullServices::default();
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());

    let snapshot1 = canvas.sync_view_state(&mut host);
    let (geom1, index1) = canvas.canvas_derived(&host, &snapshot1);
    let counters1 = canvas.debug_derived_build_counters();

    Theme::with_global_mut(&mut host, |theme| {
        let mut cfg = ThemeConfig::default();
        cfg.name = "theme_metric_updates_rebuild_canvas_geometry_in_system_mode".to_string();
        cfg.metrics = HashMap::from([("metric.padding.md".to_string(), 32.0)]);
        theme.apply_config(&cfg);
    });

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());
    let snapshot2 = canvas.sync_view_state(&mut host);
    let (geom2, index2) = canvas.canvas_derived(&host, &snapshot2);
    let counters2 = canvas.debug_derived_build_counters();

    assert!(!Arc::ptr_eq(&geom1, &geom2));
    assert!(!Arc::ptr_eq(&index1, &index2));
    assert!(
        counters2.geom_rebuilds > counters1.geom_rebuilds,
        "metric changes must rebuild derived geometry"
    );
    assert!(
        counters2.index_rebuilds > counters1.index_rebuilds,
        "metric changes must rebuild the spatial index"
    );
}
