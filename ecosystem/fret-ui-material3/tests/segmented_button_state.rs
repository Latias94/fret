#![cfg(feature = "diagnostics")]

//! Material 3 segmented button semantics, layout, parts, and state-layer tests.

use std::{collections::BTreeSet, sync::Arc};

use fret_core::{
    AppWindowId, Axis, DrawOrder, Paint, Point, PointerId, Px, Rect, Scene, SceneOp,
    SemanticsCheckedState, SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::{FlexProps, Length};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{SegmentedButtonItem, SegmentedButtonSet};

mod interaction_harness;
mod support;

use support::events::{pointer_down, pointer_move};
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(280.0)),
    )
}

fn render_segmented(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    single: Model<Arc<str>>,
    multi: Model<BTreeSet<Arc<str>>>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(16.0).into();
        column.layout.size.width = Length::Px(Px(360.0));

        let content = cx.flex(column, |cx| {
            vec![
                SegmentedButtonSet::single(single)
                    .a11y_label("Single segmented")
                    .test_id("m3-segmented-single")
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .test_id("m3-segmented-single-alpha"),
                        SegmentedButtonItem::new("beta", "Beta")
                            .test_id("m3-segmented-single-beta"),
                    ])
                    .into_element(cx),
                SegmentedButtonSet::multi(multi)
                    .a11y_label("Multi segmented")
                    .test_id("m3-segmented-multi")
                    .items(vec![
                        SegmentedButtonItem::new("alpha", "Alpha")
                            .test_id("m3-segmented-multi-alpha"),
                        SegmentedButtonItem::new("beta", "Beta").test_id("m3-segmented-multi-beta"),
                    ])
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

fn assert_px_close(actual: f32, expected: f32, label: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "expected {label} {expected}px, got {actual}px (delta {delta}px)"
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

struct SegmentedHarness {
    app: TestHost,
    window: AppWindowId,
    services: FakeUiServices,
    ui: UiTree<TestHost>,
    single: Model<Arc<str>>,
    multi: Model<BTreeSet<Arc<str>>>,
}

fn segmented_harness() -> SegmentedHarness {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let single = app.models_mut().insert(Arc::<str>::from("alpha"));
    let multi = app
        .models_mut()
        .insert([Arc::<str>::from("alpha")].into_iter().collect());

    SegmentedHarness {
        app,
        window,
        services,
        ui,
        single,
        multi,
    }
}

#[test]
fn segmented_button_semantics_expose_material_checked_state() {
    let SegmentedHarness {
        mut app,
        window,
        mut services,
        mut ui,
        single,
        multi,
    } = segmented_harness();
    render_segmented(&mut ui, &mut app, &mut services, window, single, multi);

    let single_group = semantics_node(&ui, "m3-segmented-single");
    assert_eq!(single_group.role, SemanticsRole::RadioGroup);

    let single_alpha = semantics_node(&ui, "m3-segmented-single-alpha");
    assert_eq!(single_alpha.role, SemanticsRole::RadioButton);
    assert_eq!(single_alpha.flags.checked, Some(true));
    assert_eq!(
        single_alpha.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );
    assert!(!single_alpha.flags.selected);

    let single_beta = semantics_node(&ui, "m3-segmented-single-beta");
    assert_eq!(single_beta.role, SemanticsRole::RadioButton);
    assert_eq!(single_beta.flags.checked, Some(false));
    assert_eq!(
        single_beta.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
    assert!(!single_beta.flags.selected);

    let multi_group = semantics_node(&ui, "m3-segmented-multi");
    assert_eq!(multi_group.role, SemanticsRole::Group);

    let multi_alpha = semantics_node(&ui, "m3-segmented-multi-alpha");
    assert_eq!(multi_alpha.role, SemanticsRole::Checkbox);
    assert_eq!(multi_alpha.flags.checked, Some(true));
    assert_eq!(
        multi_alpha.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );
    assert!(!multi_alpha.flags.selected);

    let multi_beta = semantics_node(&ui, "m3-segmented-multi-beta");
    assert_eq!(multi_beta.role, SemanticsRole::Checkbox);
    assert_eq!(multi_beta.flags.checked, Some(false));
    assert_eq!(
        multi_beta.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
    assert!(!multi_beta.flags.selected);
}

#[test]
fn segmented_button_segments_join_and_expose_material_parts() {
    let SegmentedHarness {
        mut app,
        window,
        mut services,
        mut ui,
        single,
        multi,
    } = segmented_harness();
    render_segmented(&mut ui, &mut app, &mut services, window, single, multi);

    let alpha_touch = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-segmented-single-alpha").id)
        .expect("expected alpha touch target bounds");
    let beta_touch = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-segmented-single-beta").id)
        .expect("expected beta touch target bounds");
    assert_size(alpha_touch, 180.0, 48.0, "alpha touch target");
    assert_size(beta_touch, 180.0, 48.0, "beta touch target");

    let alpha_chrome =
        visual_bounds_by_test_id(&ui, &app, window, "m3-segmented-single-alpha.chrome");
    let beta_chrome =
        visual_bounds_by_test_id(&ui, &app, window, "m3-segmented-single-beta.chrome");
    assert_size(alpha_chrome, 180.0, 40.0, "alpha chrome");
    assert_size(beta_chrome, 180.0, 40.0, "beta chrome");
    assert_px_close(
        alpha_chrome.origin.x.0 + alpha_chrome.size.width.0,
        beta_chrome.origin.x.0,
        "joined segment seam",
    );
    assert_px_close(
        alpha_chrome.origin.y.0,
        alpha_touch.origin.y.0 + 4.0,
        "chrome centered inside touch target",
    );

    for id in [
        "m3-segmented-single-alpha.icon",
        "m3-segmented-single-beta.icon",
    ] {
        let bounds = visual_bounds_by_test_id(&ui, &app, window, id);
        assert!(
            bounds.size.width.0 > 0.0 && bounds.size.height.0 > 0.0,
            "expected non-empty Material segmented button part {id}, got {bounds:?}"
        );
    }
    for id in [
        "m3-segmented-single-alpha.label",
        "m3-segmented-single-beta.label",
    ] {
        let bounds = visual_bounds_by_test_id(&ui, &app, window, id);
        assert!(
            bounds.size.height.0 > 0.0,
            "expected Material segmented button label part {id}, got {bounds:?}"
        );
    }
}

#[test]
fn segmented_button_pressed_state_layer_animates_over_segment_chrome() {
    let SegmentedHarness {
        mut app,
        window,
        mut services,
        mut ui,
        single,
        multi,
    } = segmented_harness();
    let multi_for_render = multi.clone();
    render_segmented(
        &mut ui,
        &mut app,
        &mut services,
        window,
        single.clone(),
        multi.clone(),
    );

    let beta_chrome =
        visual_bounds_by_test_id(&ui, &app, window, "m3-segmented-single-beta.chrome");
    assert!(
        state_layer_alphas_for_chrome(&paint(&mut ui, &mut app, &mut services), beta_chrome)
            .is_empty(),
        "idle segmented button should not paint a visible state layer"
    );

    let beta = semantics_node(&ui, "m3-segmented-single-beta");
    let beta_bounds = ui
        .debug_node_visual_bounds(beta.id)
        .expect("expected beta touch target bounds");
    let press_at = Point::new(
        Px(beta_bounds.origin.x.0 + beta_bounds.size.width.0 * 0.5),
        Px(beta_bounds.origin.y.0 + beta_bounds.size.height.0 * 0.5),
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
        render_segmented(
            &mut ui,
            &mut app,
            &mut services,
            window,
            single.clone(),
            multi_for_render.clone(),
        );
        animated.extend(state_layer_alphas_for_chrome(
            &paint(&mut ui, &mut app, &mut services),
            beta_chrome,
        ));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed segmented button state layer to animate through partial alpha, got {animated:?}"
    );
}
