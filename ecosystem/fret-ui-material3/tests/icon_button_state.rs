#![cfg(feature = "diagnostics")]

//! Material 3 icon button semantics, layout, parts, and state-layer tests.

use fret_core::{
    AppWindowId, DrawOrder, Paint, Point, PointerId, Px, Rect, Scene, SceneOp,
    SemanticsCheckedState, SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{IconButton, IconToggleButton};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_move, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    )
}

fn render_icon_buttons(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    checked: Model<bool>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut row = fret_ui::element::FlexProps::default();
        row.direction = fret_core::Axis::Horizontal;
        row.gap = Px(12.0).into();

        let content = cx.flex(row, |cx| {
            vec![
                IconButton::new(fret_icons::ids::ui::SEARCH)
                    .a11y_label("Search")
                    .test_id("m3-icon-button")
                    .into_element(cx),
                IconButton::new(fret_icons::ids::ui::SETTINGS)
                    .toggle(true)
                    .selected(true)
                    .a11y_label("Favorite")
                    .test_id("m3-icon-button-toggle-selected")
                    .into_element(cx),
                IconToggleButton::new(checked, fret_icons::ids::ui::CHECK)
                    .a11y_label("Toggle")
                    .test_id("m3-icon-toggle")
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
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
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

fn state_layer_alphas_for_chrome(scene: &Scene, chrome: Rect) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect,
                background,
                order,
                border,
                ..
            } if order == DrawOrder(0)
                && rect.origin.x.0 >= chrome.origin.x.0 - 0.1
                && rect.origin.y.0 >= chrome.origin.y.0 - 0.1
                && rect.origin.x.0 + rect.size.width.0
                    <= chrome.origin.x.0 + chrome.size.width.0 + 0.1
                && rect.origin.y.0 + rect.size.height.0
                    <= chrome.origin.y.0 + chrome.size.height.0 + 0.1
                && rect.size.width.0 >= chrome.size.width.0 - 2.1
                && rect.size.height.0 >= chrome.size.height.0 - 2.1
                && border.left.0 == 0.0
                && border.right.0 == 0.0
                && border.top.0 == 0.0
                && border.bottom.0 == 0.0 =>
            {
                match background.paint {
                    Paint::Solid(color) if color.a > 0.0 => Some(color.a),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

struct IconButtonHarness {
    app: TestHost,
    window: AppWindowId,
    services: FakeUiServices,
    ui: UiTree<TestHost>,
    checked: Model<bool>,
}

fn icon_button_harness() -> IconButtonHarness {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(false);

    IconButtonHarness {
        app,
        window,
        services,
        ui,
        checked,
    }
}

#[test]
fn icon_button_toggle_semantics_expose_material_checked_state() {
    let IconButtonHarness {
        mut app,
        window,
        mut services,
        mut ui,
        checked,
    } = icon_button_harness();
    render_icon_buttons(&mut ui, &mut app, &mut services, window, checked.clone());

    let button = semantics_node(&ui, "m3-icon-button");
    assert_eq!(button.role, SemanticsRole::Button);
    assert_eq!(button.flags.checked, None);
    assert_eq!(button.flags.checked_state, None);

    let toggle_selected = semantics_node(&ui, "m3-icon-button-toggle-selected");
    assert_eq!(toggle_selected.role, SemanticsRole::Checkbox);
    assert_eq!(toggle_selected.flags.checked, Some(true));
    assert_eq!(
        toggle_selected.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );
    assert!(!toggle_selected.flags.selected);

    let toggle = semantics_node(&ui, "m3-icon-toggle");
    assert_eq!(toggle.role, SemanticsRole::Checkbox);
    assert_eq!(toggle.flags.checked, Some(false));
    assert_eq!(
        toggle.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
    assert!(!toggle.flags.selected);

    let toggle_bounds = ui
        .debug_node_visual_bounds(toggle.id)
        .expect("expected icon toggle visual bounds");
    let press_at = Point::new(
        Px(toggle_bounds.origin.x.0 + toggle_bounds.size.width.0 * 0.5),
        Px(toggle_bounds.origin.y.0 + toggle_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), press_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    app.advance_frame();
    render_icon_buttons(&mut ui, &mut app, &mut services, window, checked);

    let toggled = semantics_node(&ui, "m3-icon-toggle");
    assert_eq!(toggled.flags.checked, Some(true));
    assert_eq!(
        toggled.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );
}

#[test]
fn icon_button_exposes_touch_chrome_and_icon_parts() {
    let IconButtonHarness {
        mut app,
        window,
        mut services,
        mut ui,
        checked,
    } = icon_button_harness();
    render_icon_buttons(&mut ui, &mut app, &mut services, window, checked);

    for id in [
        "m3-icon-button",
        "m3-icon-button-toggle-selected",
        "m3-icon-toggle",
    ] {
        let touch = ui
            .debug_node_visual_bounds(semantics_node(&ui, id).id)
            .unwrap_or_else(|| panic!("expected touch bounds for {id}"));
        assert_size(touch, 48.0, 48.0, id);

        let chrome = visual_bounds_by_test_id(&ui, &app, window, &format!("{id}.chrome"));
        assert_size(chrome, 40.0, 40.0, &format!("{id}.chrome"));

        let icon = visual_bounds_by_test_id(&ui, &app, window, &format!("{id}.icon"));
        assert_size(icon, 24.0, 24.0, &format!("{id}.icon"));
    }
}

#[test]
fn icon_button_pressed_state_layer_animates_over_chrome() {
    let IconButtonHarness {
        mut app,
        window,
        mut services,
        mut ui,
        checked,
    } = icon_button_harness();
    render_icon_buttons(&mut ui, &mut app, &mut services, window, checked.clone());

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-icon-button.chrome");
    assert!(
        state_layer_alphas_for_chrome(&paint(&mut ui, &mut app, &mut services), chrome).is_empty(),
        "idle icon button should not paint a visible state layer"
    );

    let node = semantics_node(&ui, "m3-icon-button");
    let touch = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected icon button touch bounds");
    let press_at = Point::new(
        Px(touch.origin.x.0 + touch.size.width.0 * 0.5),
        Px(touch.origin.y.0 + touch.size.height.0 * 0.5),
    );
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
        render_icon_buttons(&mut ui, &mut app, &mut services, window, checked.clone());
        animated.extend(state_layer_alphas_for_chrome(
            &paint(&mut ui, &mut app, &mut services),
            chrome,
        ));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed icon button state layer to animate through partial alpha, got {animated:?}"
    );
}
