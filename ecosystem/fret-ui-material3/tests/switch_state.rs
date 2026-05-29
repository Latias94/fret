#![cfg(feature = "diagnostics")]

//! Material 3 switch semantics, layout, and motion regression tests.

use fret_core::{
    AppWindowId, Axis, Point, PointerId, Px, Rect, SemanticsCheckedState, SemanticsNode,
    SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::FlexProps;
use fret_ui_material3::Switch;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    )
}

fn render_switch(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<bool>,
    icons: bool,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut switch = Switch::new(selected)
            .a11y_label("Switch")
            .test_id("m3-switch");
        if icons {
            switch = switch.icons(true);
        }
        let switch = switch.into_element(cx);
        vec![with_padding(cx, Px(32.0), switch)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_switch_matrix(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<bool>,
    unselected: Model<bool>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(8.0).into();

        let content = cx.flex(column, |cx| {
            vec![
                Switch::new(selected)
                    .a11y_label("Selected switch")
                    .test_id("m3-switch-selected")
                    .into_element(cx),
                Switch::new(unselected)
                    .a11y_label("Unselected switch")
                    .test_id("m3-switch-unselected")
                    .into_element(cx),
            ]
        });

        vec![with_padding(cx, Px(32.0), content)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
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

fn rect_center_x(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0 * 0.5
}

fn switch_center(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node(ui, test_id);
    let bounds = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected switch visual bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

#[test]
fn switch_semantics_expose_binary_checked_state() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(true);
    let unselected = app.models_mut().insert(false);
    render_switch_matrix(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected,
        unselected,
    );

    let selected = semantics_node(&ui, "m3-switch-selected");
    assert_eq!(selected.role, SemanticsRole::Switch);
    assert_eq!(selected.flags.checked, Some(true));
    assert_eq!(
        selected.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );

    let unselected = semantics_node(&ui, "m3-switch-unselected");
    assert_eq!(unselected.role, SemanticsRole::Switch);
    assert_eq!(unselected.flags.checked, Some(false));
    assert_eq!(
        unselected.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
}

#[test]
fn switch_exposes_touch_state_layer_track_handle_and_icon_parts() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(true);
    render_switch(&mut ui, &mut app, &mut services, window, selected, true);

    let root = semantics_node(&ui, "m3-switch");
    let root_bounds = ui
        .debug_node_visual_bounds(root.id)
        .expect("expected switch root bounds");
    assert_size(root_bounds, 52.0, 48.0, "switch touch target");
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-switch.chrome"),
        52.0,
        40.0,
        "switch state-layer chrome",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-switch.track"),
        52.0,
        32.0,
        "switch track",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-switch.handle"),
        24.0,
        24.0,
        "switch selected handle",
    );

    for id in ["m3-switch.icon-on", "m3-switch.icon-off"] {
        let _ = visual_bounds_by_test_id(&ui, &app, window, id);
    }
}

#[test]
fn switch_handle_moves_and_grows_after_toggle() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(false);
    render_switch(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        false,
    );

    let press_at = switch_center(&ui, "m3-switch");
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

    render_switch(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        false,
    );
    let initial_handle = visual_bounds_by_test_id(&ui, &app, window, "m3-switch.handle");

    for _ in 0..4 {
        app.advance_frame();
        render_switch(
            &mut ui,
            &mut app,
            &mut services,
            window,
            selected.clone(),
            false,
        );
    }
    let later_handle = visual_bounds_by_test_id(&ui, &app, window, "m3-switch.handle");

    assert!(
        rect_center_x(later_handle) > rect_center_x(initial_handle) + 4.0,
        "expected switch handle to move toward the selected edge, initial={initial_handle:?}, later={later_handle:?}"
    );
    assert!(
        later_handle.size.width.0 >= initial_handle.size.width.0,
        "expected switch handle to grow or hold size during selection motion, initial={initial_handle:?}, later={later_handle:?}"
    );
}
