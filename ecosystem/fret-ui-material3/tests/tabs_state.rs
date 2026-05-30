#![cfg(feature = "diagnostics")]

//! Material 3 Tabs semantics and layout parity tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, Paint, Point, Px, Rect, Scene, SceneOp, SemanticsNode, SemanticsOrientation,
    SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{TabItem, Tabs, TabsVariant};

mod interaction_harness;
mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    )
}

fn tabs_harness() -> (
    TestHost,
    AppWindowId,
    FakeUiServices,
    UiTree<TestHost>,
    Model<Arc<str>>,
) {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(Arc::<str>::from("a"));
    (app, window, services, ui, selected)
}

fn render_tabs(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
    scrollable: bool,
) {
    render_tabs_with_variant(
        ui,
        app,
        services,
        window,
        selected,
        scrollable,
        TabsVariant::Primary,
    );
}

fn render_tabs_with_variant(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
    scrollable: bool,
    variant: TabsVariant,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let tabs = Tabs::new(selected)
                .a11y_label("Material tabs")
                .test_id("m3-tabs")
                .scrollable(scrollable)
                .variant(variant)
                .items(vec![
                    TabItem::new("a", "A").test_id("m3-tab-a"),
                    TabItem::new("b", "B").test_id("m3-tab-b"),
                    TabItem::new("disabled", "Disabled")
                        .disabled(true)
                        .test_id("m3-tab-disabled"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), tabs)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn paint(ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds(), &mut scene, 1.0);
    scene
}

fn settle_tabs(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
    scrollable: bool,
) -> Scene {
    settle_tabs_with_variant(
        ui,
        app,
        services,
        window,
        selected,
        scrollable,
        TabsVariant::Primary,
    )
}

fn settle_tabs_with_variant(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
    scrollable: bool,
    variant: TabsVariant,
) -> Scene {
    let mut scene = Scene::default();
    for _ in 0..6 {
        render_tabs_with_variant(
            ui,
            app,
            services,
            window,
            selected.clone(),
            scrollable,
            variant,
        );
        scene = paint(ui, app, services);
        app.advance_frame();
    }
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

fn active_indicator_rect(scene: &Scene) -> Rect {
    let quad_summary: Vec<String> = scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } => Some(format!(
                "quad x={:.1} y={:.1} w={:.1} h={:.1} alpha={:.3}",
                rect.origin.x.0,
                rect.origin.y.0,
                rect.size.width.0,
                rect.size.height.0,
                match background.paint {
                    Paint::Solid(color) => color.a,
                    _ => -1.0,
                }
            )),
            _ => None,
        })
        .collect();
    let mut candidates: Vec<Rect> = scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } if (2.5..=3.5).contains(&rect.size.height.0)
                && rect.size.width.0 > 0.0
                && matches!(background.paint, Paint::Solid(color) if color.a > 0.0) =>
            {
                Some(rect)
            }
            _ => None,
        })
        .collect();
    candidates.sort_by(|a, b| b.origin.y.0.total_cmp(&a.origin.y.0));
    candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("expected active indicator quad; quads={quad_summary:?}"))
}

#[test]
fn tabs_export_tablist_orientation_and_tab_collection_semantics() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    render_tabs(&mut ui, &mut app, &mut services, window, selected, false);

    let list = semantics_node(&ui, "m3-tabs");
    assert_eq!(list.role, SemanticsRole::TabList);
    assert_eq!(
        list.extra.orientation,
        Some(SemanticsOrientation::Horizontal)
    );
    assert_eq!(list.label.as_deref(), Some("Material tabs"));

    let a = semantics_node(&ui, "m3-tab-a");
    assert_eq!(a.role, SemanticsRole::Tab);
    assert!(a.flags.selected);
    assert_eq!(a.pos_in_set, Some(1));
    assert_eq!(a.set_size, Some(3));

    let b = semantics_node(&ui, "m3-tab-b");
    assert_eq!(b.role, SemanticsRole::Tab);
    assert!(!b.flags.selected);
    assert_eq!(b.pos_in_set, Some(2));
    assert_eq!(b.set_size, Some(3));

    let disabled = semantics_node(&ui, "m3-tab-disabled");
    assert_eq!(disabled.role, SemanticsRole::Tab);
    assert!(disabled.flags.disabled);
    assert_eq!(disabled.pos_in_set, Some(3));
    assert_eq!(disabled.set_size, Some(3));
}

#[test]
fn fixed_primary_tabs_use_content_sized_active_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_tabs(&mut ui, &mut app, &mut services, window, selected, false);

    let tab = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-tab-a").id)
        .expect("expected tab visual bounds");
    let indicator_canvas = visual_bounds_by_test_id(&ui, &app, window, "m3-tabs.active-indicator");
    assert!(
        indicator_canvas.size.width.0 > 0.0 && indicator_canvas.size.height.0 > 0.0,
        "expected active indicator canvas to fill the tab row, got {indicator_canvas:?}"
    );
    let indicator = active_indicator_rect(&scene);

    assert!(
        (indicator.size.width.0 - 24.0).abs() <= 0.5,
        "expected short-label primary tab indicator to use the 24px Material minimum, got {:?}",
        indicator
    );
    assert!(
        (indicator.origin.x.0 + indicator.size.width.0 * 0.5
            - (tab.origin.x.0 + tab.size.width.0 * 0.5))
            .abs()
            <= 0.5,
        "expected primary tab indicator to be centered under the selected tab content; tab={tab:?} indicator={indicator:?}"
    );
}

#[test]
fn fixed_secondary_tabs_use_full_width_active_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_tabs_with_variant(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected,
        false,
        TabsVariant::Secondary,
    );

    let tab = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-tab-a").id)
        .expect("expected tab visual bounds");
    let indicator = active_indicator_rect(&scene);

    assert!(
        (indicator.origin.x.0 - tab.origin.x.0).abs() <= 0.5,
        "expected secondary tab indicator to start at the selected tab edge; tab={tab:?} indicator={indicator:?}"
    );
    assert!(
        (indicator.size.width.0 - tab.size.width.0).abs() <= 0.5,
        "expected secondary tab indicator to span the selected tab width; tab={tab:?} indicator={indicator:?}"
    );
}

#[test]
fn scrollable_primary_tabs_use_material_edge_padding_and_min_width() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let _scene = settle_tabs(&mut ui, &mut app, &mut services, window, selected, true);

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-tabs.chrome");
    let tab = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-tab-a").id)
        .expect("expected tab visual bounds");

    assert!(
        (tab.origin.x.0 - (chrome.origin.x.0 + 52.0)).abs() <= 0.5,
        "expected scrollable primary tabs to start after Material 52px edge padding; chrome={chrome:?} tab={tab:?}"
    );
    assert!(
        (tab.size.width.0 - 90.0).abs() <= 0.5,
        "expected scrollable primary tab to use Material 90px min width, got {:?}",
        tab
    );
}

#[test]
fn scrollable_secondary_tabs_use_material_metrics_and_full_width_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_tabs_with_variant(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected,
        true,
        TabsVariant::Secondary,
    );

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-tabs.chrome");
    let tab = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-tab-a").id)
        .expect("expected tab visual bounds");
    let indicator = active_indicator_rect(&scene);

    assert!(
        (tab.origin.x.0 - (chrome.origin.x.0 + 52.0)).abs() <= 0.5,
        "expected scrollable secondary tabs to start after Material 52px edge padding; chrome={chrome:?} tab={tab:?}"
    );
    assert!(
        (tab.size.width.0 - 90.0).abs() <= 0.5,
        "expected scrollable secondary tab to use Material 90px min width, got {:?}",
        tab
    );
    assert!(
        (indicator.origin.x.0 - tab.origin.x.0).abs() <= 0.5,
        "expected scrollable secondary indicator to start at the selected tab edge; tab={tab:?} indicator={indicator:?}"
    );
    assert!(
        (indicator.size.width.0 - tab.size.width.0).abs() <= 0.5,
        "expected scrollable secondary indicator to span the selected tab width; tab={tab:?} indicator={indicator:?}"
    );
}
