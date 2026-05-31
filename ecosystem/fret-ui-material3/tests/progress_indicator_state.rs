#![cfg(feature = "diagnostics")]

//! Material 3 progress indicator semantics and motion regression tests.

use fret_core::{
    AppWindowId, Axis, Point, Px, Rect, Scene, SemanticsRole, Size, UiServices,
    semantics::SemanticsNode,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::FlexProps;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{CircularProgressIndicator, LinearProgressIndicator};

mod support;

use support::host::{FakeUiServices, TestHost};
use support::interaction_harness::scene_quad_geometry_signature;
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(260.0)),
    )
}

fn render_progress(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    linear_progress: fret_runtime::Model<f32>,
    circular_progress: fret_runtime::Model<f32>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(16.0).into();

        let content = cx.flex(column, |cx| {
            vec![
                LinearProgressIndicator::new(linear_progress)
                    .a11y_label("Linear progress")
                    .test_id("m3-linear-progress")
                    .into_element(cx),
                CircularProgressIndicator::new(circular_progress)
                    .a11y_label("Circular progress")
                    .test_id("m3-circular-progress")
                    .into_element(cx),
                LinearProgressIndicator::indeterminate()
                    .a11y_label("Linear loading")
                    .test_id("m3-linear-indeterminate")
                    .into_element(cx),
                CircularProgressIndicator::indeterminate()
                    .a11y_label("Circular loading")
                    .test_id("m3-circular-indeterminate")
                    .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_indeterminate(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(24.0).into();

        let content = cx.flex(column, |cx| {
            vec![
                LinearProgressIndicator::indeterminate()
                    .test_id("linear-indeterminate")
                    .into_element(cx),
                CircularProgressIndicator::indeterminate()
                    .test_id("circular-indeterminate")
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

fn live_test_id_exists(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> bool {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .any(|m| {
            ui.debug_node_visual_bounds(m.node).is_some() || ui.debug_node_bounds(m.node).is_some()
        })
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

fn assert_progressbar_value(node: &SemanticsNode, expected: f64) {
    assert_eq!(node.role, SemanticsRole::ProgressBar);
    assert_eq!(node.extra.numeric.min, Some(0.0));
    assert_eq!(node.extra.numeric.max, Some(1.0));
    let actual = node
        .extra
        .numeric
        .value
        .expect("expected determinate progress value");
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected progress value {expected}, got {actual}"
    );
    assert!(!node.flags.busy);
}

#[test]
fn determinate_and_indeterminate_progress_expose_progressbar_semantics() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let linear = app.models_mut().insert(0.3f32);
    let circular = app.models_mut().insert(0.7f32);
    render_progress(&mut ui, &mut app, &mut services, window, linear, circular);

    let linear = semantics_node(&ui, "m3-linear-progress");
    assert_eq!(linear.label.as_deref(), Some("Linear progress"));
    assert_progressbar_value(linear, 0.3);

    let circular = semantics_node(&ui, "m3-circular-progress");
    assert_eq!(circular.label.as_deref(), Some("Circular progress"));
    assert_progressbar_value(circular, 0.7);

    for (id, label) in [
        ("m3-linear-indeterminate", "Linear loading"),
        ("m3-circular-indeterminate", "Circular loading"),
    ] {
        let node = semantics_node(&ui, id);
        assert_eq!(node.role, SemanticsRole::ProgressBar);
        assert_eq!(node.label.as_deref(), Some(label));
        assert!(node.flags.busy);
        assert_eq!(node.extra.numeric.value, None);
        assert_eq!(node.extra.numeric.min, None);
        assert_eq!(node.extra.numeric.max, None);
    }
}

#[test]
fn progress_indicators_expose_stable_track_part_ids() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let linear = app.models_mut().insert(0.3f32);
    let circular = app.models_mut().insert(0.7f32);
    render_progress(&mut ui, &mut app, &mut services, window, linear, circular);

    for id in [
        "m3-linear-progress.track",
        "m3-linear-progress.active-track",
        "m3-circular-progress.track",
        "m3-circular-progress.active-track",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected stable progress indicator part test_id {id}"
        );
    }
}

#[test]
fn indeterminate_progress_indicators_advance_draw_region_between_frames() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_indeterminate(&mut ui, &mut app, &mut services, window);
    let initial = scene_quad_geometry_signature(&paint(&mut ui, &mut app, &mut services));

    for _ in 0..30 {
        app.advance_frame();
        render_indeterminate(&mut ui, &mut app, &mut services, window);
    }
    let later = scene_quad_geometry_signature(&paint(&mut ui, &mut app, &mut services));

    assert_ne!(
        initial, later,
        "expected indeterminate progress draw regions to move between frames"
    );
}
