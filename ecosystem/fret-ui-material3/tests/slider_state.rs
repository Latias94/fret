#![cfg(feature = "diagnostics")]

//! Material 3 slider semantics, draw-region, and state-layer regression tests.

use fret_core::{
    AppWindowId, Axis, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsNode, SemanticsRole,
    Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::{FlexProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{RangeSlider, Slider};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_move, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(280.0)),
    )
}

fn render_slider(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    value: Model<f32>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.layout.size.width = Length::Px(Px(360.0));

        let content = cx.flex(column, |cx| {
            vec![
                Slider::new(value)
                    .range(0.0, 1.0)
                    .a11y_label("Slider")
                    .test_id("m3-slider")
                    .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_range_slider(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    values: Model<[f32; 2]>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.layout.size.width = Length::Px(Px(360.0));

        let content = cx.flex(column, |cx| {
            vec![
                RangeSlider::new(values)
                    .range(0.0, 1.0)
                    .a11y_label("Range")
                    .test_id("m3-range-slider")
                    .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn paint(ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds(), &mut scene, 1.0);
    scene
}

fn semantics_node<'a>(ui: &'a UiTree<TestHost>, test_id: &str) -> &'a SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
}

fn visual_bounds_by_test_id(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    test_id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, test_id)
        .into_iter()
        .find_map(|m| ui.debug_node_visual_bounds(m.node))
        .unwrap_or_else(|| panic!("expected visual bounds for test_id {test_id}"))
}

fn assert_numeric(actual: Option<f64>, expected: f64, label: &str) {
    let actual = actual.unwrap_or_else(|| panic!("expected numeric {label}"));
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected numeric {label} {expected}, got {actual}"
    );
}

fn assert_size(rect: Rect, width: f32, height: f32, label: &str) {
    assert!(
        (rect.size.width.0 - width).abs() <= 0.01,
        "expected {label} width {width}, got {}",
        rect.size.width.0
    );
    assert!(
        (rect.size.height.0 - height).abs() <= 0.01,
        "expected {label} height {height}, got {}",
        rect.size.height.0
    );
}

fn state_layer_alphas(scene: &Scene) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } if (rect.size.width.0 - 40.0).abs() <= 0.1
                && (rect.size.height.0 - 40.0).abs() <= 0.1 =>
            {
                match background.paint {
                    fret_core::Paint::Solid(color) => Some(color.a),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

fn slider_center_at(ui: &UiTree<TestHost>, test_id: &str, t: f32) -> Point {
    let node = semantics_node(ui, test_id);
    let bounds = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected slider visual bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * t.clamp(0.0, 1.0)),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

#[test]
fn slider_semantics_expose_continuous_numeric_actions() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let value = app.models_mut().insert(0.4_f32);
    render_slider(&mut ui, &mut app, &mut services, window, value);

    let slider = semantics_node(&ui, "m3-slider");
    assert_eq!(slider.role, SemanticsRole::Slider);
    assert_eq!(slider.label.as_deref(), Some("Slider"));
    assert_eq!(slider.value.as_deref(), Some("0.400"));
    assert_numeric(slider.extra.numeric.value, 0.4, "value");
    assert_numeric(slider.extra.numeric.min, 0.0, "min");
    assert_numeric(slider.extra.numeric.max, 1.0, "max");
    assert_numeric(slider.extra.numeric.step, 0.01, "step");
    assert_numeric(slider.extra.numeric.jump, 0.1, "jump");
    assert!(slider.actions.increment);
    assert!(slider.actions.decrement);
    assert!(slider.actions.set_value);
}

#[test]
fn range_slider_thumb_semantics_are_constrained_by_peer_thumb() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let values = app.models_mut().insert([0.3_f32, 0.7_f32]);
    render_range_slider(&mut ui, &mut app, &mut services, window, values);

    let group = semantics_node(&ui, "m3-range-slider");
    assert_eq!(group.role, SemanticsRole::Group);
    assert_eq!(group.label.as_deref(), Some("Range"));
    assert_eq!(group.value.as_deref(), Some("0.300..0.700"));

    let start = semantics_node(&ui, "m3-range-slider.start");
    assert_eq!(start.role, SemanticsRole::Slider);
    assert_eq!(start.label.as_deref(), Some("Range start"));
    assert_numeric(start.extra.numeric.value, 0.3, "start value");
    assert_numeric(start.extra.numeric.min, 0.0, "start min");
    assert_numeric(start.extra.numeric.max, 0.7, "start max");
    assert_numeric(start.extra.numeric.step, 0.01, "start step");
    assert_numeric(start.extra.numeric.jump, 0.1, "start jump");
    assert!(start.actions.set_value);

    let end = semantics_node(&ui, "m3-range-slider.end");
    assert_eq!(end.role, SemanticsRole::Slider);
    assert_eq!(end.label.as_deref(), Some("Range end"));
    assert_numeric(end.extra.numeric.value, 0.7, "end value");
    assert_numeric(end.extra.numeric.min, 0.3, "end min");
    assert_numeric(end.extra.numeric.max, 1.0, "end max");
    assert_numeric(end.extra.numeric.step, 0.01, "end step");
    assert_numeric(end.extra.numeric.jump, 0.1, "end jump");
    assert!(end.actions.set_value);
}

#[test]
fn slider_active_track_and_handle_bounds_follow_pointer_drag() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let value = app.models_mut().insert(0.25_f32);
    render_slider(&mut ui, &mut app, &mut services, window, value.clone());

    let track = visual_bounds_by_test_id(&ui, &app, window, "m3-slider.track");
    assert_size(track, 360.0, 16.0, "slider track");
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-slider.handle"),
        4.0,
        44.0,
        "slider handle",
    );
    let active_before = visual_bounds_by_test_id(&ui, &app, window, "m3-slider.active-track");

    let press_at = slider_center_at(&ui, "m3-slider", 0.75);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    render_slider(&mut ui, &mut app, &mut services, window, value.clone());

    let value_now = app
        .models_mut()
        .read(&value, |value| *value)
        .expect("expected slider value");
    assert!(
        value_now > 0.70,
        "expected pointer drag to update slider value, got {value_now}"
    );

    let active_after = visual_bounds_by_test_id(&ui, &app, window, "m3-slider.active-track");
    assert!(
        active_after.size.width.0 > active_before.size.width.0 + 120.0,
        "expected active draw region to grow after drag, before={active_before:?}, after={active_after:?}"
    );
}

#[test]
fn slider_state_layer_animates_after_press() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let value = app.models_mut().insert(0.5_f32);
    render_slider(&mut ui, &mut app, &mut services, window, value.clone());
    assert!(
        state_layer_alphas(&paint(&mut ui, &mut app, &mut services)).is_empty(),
        "idle slider should not paint a visible state layer"
    );

    let press_at = slider_center_at(&ui, "m3-slider", 0.5);
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), press_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );

    let mut animated = Vec::new();
    for _ in 0..4 {
        app.advance_frame();
        render_slider(&mut ui, &mut app, &mut services, window, value.clone());
        animated.extend(state_layer_alphas(&paint(&mut ui, &mut app, &mut services)));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed slider state layer to animate through a partial alpha, got {animated:?}"
    );
}
