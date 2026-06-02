#![cfg(feature = "diagnostics")]

//! Material 3 radio semantics, layout, and motion regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, Axis, Edges, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsCheckedState,
    SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::FlexProps;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{Radio, RadioGroup, RadioGroupItem, RadioGroupOrientation};

mod support;

use support::events::{pointer_down, pointer_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::{paint_alpha, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    )
}

fn render_radio(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<bool>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let radio = Radio::new(selected)
            .a11y_label("Radio")
            .test_id("m3-radio")
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), radio)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_radio_group(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Option<Arc<str>>>,
) {
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let mut column = FlexProps::default();
        column.direction = Axis::Vertical;
        column.gap = Px(8.0).into();

        let content = cx.flex(column, |cx| {
            vec![
                RadioGroup::new(selected)
                    .orientation(RadioGroupOrientation::Horizontal)
                    .gap(Px(8.0))
                    .a11y_label("Radio group")
                    .test_id("m3-radio-group")
                    .items(vec![
                        RadioGroupItem::new("alpha")
                            .a11y_label("Alpha")
                            .test_id("m3-radio-alpha"),
                        RadioGroupItem::new("beta")
                            .a11y_label("Beta")
                            .test_id("m3-radio-beta"),
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

fn radio_center(ui: &UiTree<TestHost>, test_id: &str) -> Point {
    let node = semantics_node(ui, test_id);
    let bounds = ui
        .debug_node_visual_bounds(node.id)
        .expect("expected radio visual bounds");
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

fn largest_radio_dot_diameter(scene: &Scene) -> Option<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match op {
            SceneOp::Quad {
                rect,
                background,
                border,
                ..
            } if *border == Edges::all(Px(0.0))
                && paint_alpha(&background.paint) > 0.5
                && rect.size.width.0 > 0.05
                && rect.size.width.0 <= 12.0
                && rect.size.height.0 <= 12.0 =>
            {
                Some(rect.size.width.0)
            }
            _ => None,
        })
        .max_by(|a, b| a.total_cmp(b))
}

#[test]
fn radio_group_items_expose_checked_state_and_collection_metadata() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(Some(Arc::<str>::from("alpha")));
    render_radio_group(&mut ui, &mut app, &mut services, window, selected);

    let group = semantics_node(&ui, "m3-radio-group");
    assert_eq!(group.role, SemanticsRole::RadioGroup);

    let alpha = semantics_node(&ui, "m3-radio-alpha");
    assert_eq!(alpha.role, SemanticsRole::RadioButton);
    assert_eq!(alpha.flags.checked, Some(true));
    assert_eq!(alpha.flags.checked_state, Some(SemanticsCheckedState::True));
    assert_eq!(alpha.pos_in_set, Some(1));
    assert_eq!(alpha.set_size, Some(2));

    let beta = semantics_node(&ui, "m3-radio-beta");
    assert_eq!(beta.role, SemanticsRole::RadioButton);
    assert_eq!(beta.flags.checked, Some(false));
    assert_eq!(beta.flags.checked_state, Some(SemanticsCheckedState::False));
    assert_eq!(beta.pos_in_set, Some(2));
    assert_eq!(beta.set_size, Some(2));
}

#[test]
fn radio_exposes_touch_state_layer_icon_and_dot_geometry() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(true);
    render_radio(&mut ui, &mut app, &mut services, window, selected);

    let root = semantics_node(&ui, "m3-radio");
    let root_bounds = ui
        .debug_node_visual_bounds(root.id)
        .expect("expected radio root bounds");
    assert_size(root_bounds, 48.0, 48.0, "radio touch target");
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-radio.chrome"),
        40.0,
        40.0,
        "radio state layer",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-radio.icon"),
        20.0,
        20.0,
        "radio icon",
    );
    assert_size(
        visual_bounds_by_test_id(&ui, &app, window, "m3-radio.dot"),
        10.0,
        10.0,
        "radio selected dot",
    );
}

#[test]
fn radio_initial_selected_dot_starts_settled_and_toggle_animates() {
    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let selected = app.models_mut().insert(true);
        render_radio(&mut ui, &mut app, &mut services, window, selected);
        let settled_dot = largest_radio_dot_diameter(&paint(&mut ui, &mut app, &mut services))
            .expect("expected initially selected radio dot");
        assert!(
            (settled_dot - 10.0).abs() <= 0.1,
            "expected initially selected radio dot to start settled at 10px, got {settled_dot}"
        );
    }

    {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let selected = app.models_mut().insert(false);
        render_radio(&mut ui, &mut app, &mut services, window, selected.clone());
        let press_at = radio_center(&ui, "m3-radio");
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));

        render_radio(&mut ui, &mut app, &mut services, window, selected.clone());
        let initial_dot =
            largest_radio_dot_diameter(&paint(&mut ui, &mut app, &mut services)).unwrap_or(0.0);

        for _ in 0..4 {
            app.advance_frame();
            render_radio(&mut ui, &mut app, &mut services, window, selected.clone());
        }
        let later_dot = largest_radio_dot_diameter(&paint(&mut ui, &mut app, &mut services))
            .expect("expected radio dot after selection advances");

        assert!(
            initial_dot < 9.9,
            "expected toggled radio dot to start before its settled size, got {initial_dot}"
        );
        assert!(
            later_dot > initial_dot + 0.05,
            "expected radio dot to grow across frames, initial={initial_dot}, later={later_dot}"
        );
    }
}
