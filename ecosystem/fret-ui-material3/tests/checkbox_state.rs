#![cfg(feature = "diagnostics")]

//! Material 3 checkbox semantics, layout, and motion regression tests.

use fret_core::{
    AppWindowId, Axis, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsCheckedState,
    SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::FlexProps;
use fret_ui_material3::Checkbox;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{pointer_down, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(240.0)),
    )
}

fn render_checkbox(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    checked: Model<bool>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let checkbox = Checkbox::new(checked)
            .a11y_label("Checkbox")
            .test_id("m3-checkbox")
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), checkbox)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_checkbox_matrix(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    checked: Model<bool>,
    unchecked: Model<bool>,
    mixed: Model<Option<bool>>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(8.0).into();

        let content = cx.flex(column, |cx| {
            vec![
                Checkbox::new(checked)
                    .a11y_label("Checked")
                    .test_id("m3-checkbox-checked")
                    .into_element(cx),
                Checkbox::new(unchecked)
                    .a11y_label("Unchecked")
                    .test_id("m3-checkbox-unchecked")
                    .into_element(cx),
                Checkbox::new_optional(mixed)
                    .a11y_label("Mixed")
                    .test_id("m3-checkbox-mixed")
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

fn checkbox_center(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node(ui, test_id);
    let bounds = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected checkbox visual bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn opacity_values(scene: &Scene) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::PushOpacity { opacity } => Some(opacity),
            _ => None,
        })
        .collect()
}

fn checkbox_mark_opacity(scene: &Scene) -> f32 {
    opacity_values(scene)
        .into_iter()
        .find(|opacity| *opacity >= 0.0 && *opacity <= 1.0)
        .expect("expected animated checkbox mark opacity")
}

#[test]
fn checkbox_semantics_expose_binary_and_mixed_checked_state() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(true);
    let unchecked = app.models_mut().insert(false);
    let mixed = app.models_mut().insert(None::<bool>);
    render_checkbox_matrix(
        &mut ui,
        &mut app,
        &mut services,
        window,
        checked,
        unchecked,
        mixed,
    );

    let checked = semantics_node(&ui, "m3-checkbox-checked");
    assert_eq!(checked.role, SemanticsRole::Checkbox);
    assert_eq!(checked.flags.checked, Some(true));
    assert_eq!(
        checked.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );

    let unchecked = semantics_node(&ui, "m3-checkbox-unchecked");
    assert_eq!(unchecked.role, SemanticsRole::Checkbox);
    assert_eq!(unchecked.flags.checked, Some(false));
    assert_eq!(
        unchecked.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );

    let mixed = semantics_node(&ui, "m3-checkbox-mixed");
    assert_eq!(mixed.role, SemanticsRole::Checkbox);
    assert_eq!(mixed.flags.checked, None);
    assert_eq!(
        mixed.flags.checked_state,
        Some(SemanticsCheckedState::Mixed)
    );
}

#[test]
fn checkbox_exposes_touch_state_layer_box_and_mark_geometry() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(true);
    render_checkbox(&mut ui, &mut app, &mut services, window, checked);

    let root = semantics_node(&ui, "m3-checkbox");
    let root_bounds = ui
        .debug_node_visual_bounds(root.id)
        .expect("expected checkbox root bounds");
    assert_size(root_bounds, 48.0, 48.0, "checkbox touch target");
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-checkbox.chrome"),
        40.0,
        40.0,
        "checkbox state layer",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-checkbox.box"),
        18.0,
        18.0,
        "checkbox box",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-checkbox.mark"),
        18.0,
        18.0,
        "checkbox mark",
    );
}

#[test]
fn checkbox_checked_mark_animates_after_toggle() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(false);
    render_checkbox(&mut ui, &mut app, &mut services, window, checked.clone());

    let press_at = checkbox_center(&ui, "m3-checkbox");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    render_checkbox(&mut ui, &mut app, &mut services, window, checked.clone());
    app.advance_frame();
    render_checkbox(&mut ui, &mut app, &mut services, window, checked.clone());
    let initial_opacity = checkbox_mark_opacity(&paint(&mut ui, &mut app, &mut services));

    for _ in 0..4 {
        app.advance_frame();
        render_checkbox(&mut ui, &mut app, &mut services, window, checked.clone());
    }
    let later_opacity = checkbox_mark_opacity(&paint(&mut ui, &mut app, &mut services));

    assert!(
        initial_opacity < 0.9,
        "expected checkbox mark to start before its settled opacity, got {initial_opacity}"
    );
    assert!(
        later_opacity > initial_opacity + 0.05,
        "expected checkbox mark opacity to increase across frames, initial={initial_opacity}, later={later_opacity}"
    );
}
