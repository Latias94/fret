#![cfg(feature = "diagnostics")]

//! Material 3 Tabs semantics and layout parity tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, Paint, Point, Px, Rect, Scene, SceneOp, SemanticsNode,
    SemanticsOrientation, SemanticsRole, Size, UiServices,
};
use fret_icons::ids;
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{TabItem, TabPanel, Tabs, TabsVariant};

mod support;

use support::events::key_down;
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::{apply_material_theme, apply_material_theme_rtl};

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
    render_tabs_with_variant_and_loop_navigation(
        ui, app, services, window, selected, scrollable, variant, true,
    );
}

fn render_tabs_with_variant_and_loop_navigation(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
    scrollable: bool,
    variant: TabsVariant,
    loop_navigation: bool,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let tabs = Tabs::new(selected)
                .a11y_label("Material tabs")
                .test_id("m3-tabs")
                .scrollable(scrollable)
                .variant(variant)
                .loop_navigation(loop_navigation)
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

fn render_tabs_with_panels(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let tabs = Tabs::new(selected)
                .a11y_label("Material tabs")
                .test_id("m3-tabs")
                .items(vec![
                    TabItem::new("a", "A").test_id("m3-tab-a"),
                    TabItem::new("b", "B").test_id("m3-tab-b"),
                    TabItem::new("disabled", "Disabled")
                        .disabled(true)
                        .test_id("m3-tab-disabled"),
                ])
                .panels(vec![
                    TabPanel::new("a", [cx.text("A panel")]).test_id("m3-tab-panel-a"),
                    TabPanel::new("b", [cx.text("B panel")]).test_id("m3-tab-panel-b"),
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

fn render_leading_icon_tabs(
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
                    TabItem::new("a", "Search")
                        .leading_icon(ids::ui::SEARCH)
                        .test_id("m3-tab-a"),
                    TabItem::new("b", "Settings")
                        .leading_icon(ids::ui::SETTINGS)
                        .test_id("m3-tab-b"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), tabs)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn settle_leading_icon_tabs(
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
        render_leading_icon_tabs(
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

fn render_stacked_icon_tabs(
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
                    TabItem::new("a", "Workspace Settings")
                        .stacked_icon(ids::ui::SEARCH)
                        .test_id("m3-tab-a"),
                    TabItem::new("b", "History")
                        .stacked_icon(ids::ui::SETTINGS)
                        .test_id("m3-tab-b"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), tabs)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn settle_stacked_icon_tabs(
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
        render_stacked_icon_tabs(
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

fn layout_bounds_by_test_id(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    test_id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, test_id)
        .into_iter()
        .find_map(|m| ui.debug_node_bounds(m.node))
        .unwrap_or_else(|| panic!("expected layout bounds for test_id {test_id}"))
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
fn tabs_render_active_tab_panel_semantics_and_relations() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    render_tabs_with_panels(&mut ui, &mut app, &mut services, window, selected.clone());

    let a = semantics_node(&ui, "m3-tab-a");
    let panel = semantics_node(&ui, "m3-tab-panel-a");
    assert_eq!(panel.role, SemanticsRole::TabPanel);
    assert_eq!(panel.label.as_deref(), Some("A"));
    assert!(
        panel.labelled_by.contains(&a.id),
        "active Material tabpanel should be labelled by the selected tab"
    );
    assert!(
        a.controls.contains(&panel.id),
        "selected Material tab should expose the derived controls edge to the active tabpanel"
    );
    assert!(
        ui.semantics_snapshot()
            .expect("semantics snapshot")
            .nodes
            .iter()
            .all(|node| node.test_id.as_deref() != Some("m3-tab-panel-b")),
        "inactive non-force-mounted Material tabpanel should not be present"
    );

    app.models_mut()
        .update(&selected, |value| *value = Arc::<str>::from("b"))
        .expect("selected model should update");
    render_tabs_with_panels(&mut ui, &mut app, &mut services, window, selected);

    let b = semantics_node(&ui, "m3-tab-b");
    let panel = semantics_node(&ui, "m3-tab-panel-b");
    assert_eq!(panel.role, SemanticsRole::TabPanel);
    assert_eq!(panel.label.as_deref(), Some("B"));
    assert!(panel.labelled_by.contains(&b.id));
    assert!(b.controls.contains(&panel.id));
}

#[test]
fn rtl_tabs_arrow_left_moves_to_next_logical_tab_without_wrapping() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    render_tabs_with_variant_and_loop_navigation(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        false,
        TabsVariant::Primary,
        false,
    );

    let first = semantics_node(&ui, "m3-tab-a").id;
    let second = semantics_node(&ui, "m3-tab-b").id;
    ui.set_focus(Some(first));

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));

    assert_eq!(
        ui.focus(),
        Some(second),
        "expected RTL ArrowLeft to move forward to the next logical tab"
    );
    assert_eq!(app.models().get_cloned(&selected).as_deref(), Some("b"));
}

#[test]
fn rtl_tabs_theme_direction_mirrors_physical_tab_order() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    render_tabs(&mut ui, &mut app, &mut services, window, selected, false);

    let first = visual_bounds_by_test_id(&ui, &app, window, "m3-tab-a");
    let second = visual_bounds_by_test_id(&ui, &app, window, "m3-tab-b");

    assert!(
        first.origin.x.0 > second.origin.x.0,
        "expected RTL tab row to place the first logical tab to the physical right of the second logical tab, first={first:?}, second={second:?}"
    );
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
fn fixed_primary_tabs_render_bottom_divider_under_active_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_tabs(&mut ui, &mut app, &mut services, window, selected, false);

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-tabs.chrome");
    let divider = layout_bounds_by_test_id(&ui, &app, window, "m3-tabs.divider");
    let indicator = active_indicator_rect(&scene);

    assert!(
        (divider.size.height.0 - 1.0).abs() <= 0.5,
        "expected TabRow divider to use Material HorizontalDivider thickness, got {divider:?}"
    );
    assert!(
        (divider.origin.x.0 - chrome.origin.x.0).abs() <= 0.5
            && (divider.size.width.0 - chrome.size.width.0).abs() <= 0.5,
        "expected TabRow divider to span the row width; chrome={chrome:?} divider={divider:?}"
    );
    assert!(
        (divider.origin.y.0 + divider.size.height.0 - (chrome.origin.y.0 + chrome.size.height.0))
            .abs()
            <= 0.5,
        "expected TabRow divider at the row bottom; chrome={chrome:?} divider={divider:?}"
    );
    assert!(
        (indicator.origin.y.0 + indicator.size.height.0
            - (divider.origin.y.0 + divider.size.height.0))
            .abs()
            <= 0.5,
        "expected active indicator to share the bottom edge with the divider; indicator={indicator:?} divider={divider:?}"
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
fn primary_leading_icon_tabs_use_material_icon_size_and_content_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_leading_icon_tabs(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected,
        false,
        TabsVariant::Primary,
    );

    let icon = layout_bounds_by_test_id(&ui, &app, window, "m3-tab-a.icon");
    let label = layout_bounds_by_test_id(&ui, &app, window, "m3-tab-a.label");
    let indicator = active_indicator_rect(&scene);
    let icon_label_gap = label.origin.x.0 - (icon.origin.x.0 + icon.size.width.0);
    let expected_indicator_width = icon.size.width.0 + 8.0 + label.size.width.0;

    assert!(
        (icon.size.width.0 - 24.0).abs() <= 0.5 && (icon.size.height.0 - 24.0).abs() <= 0.5,
        "expected leading tab icon to use the Material 24px icon size, got {icon:?}"
    );
    assert!(
        icon.origin.x.0 < label.origin.x.0,
        "expected leading icon to be placed before the tab label; icon={icon:?} label={label:?}"
    );
    assert!(
        (icon_label_gap - 8.0).abs() <= 0.5,
        "expected leading icon and label to use the Material 8px gap; icon={icon:?} label={label:?}"
    );
    assert!(
        (indicator.origin.x.0 - icon.origin.x.0).abs() <= 1.5
            && (indicator.size.width.0 - expected_indicator_width).abs() <= 2.5,
        "expected primary leading-icon tab indicator to match icon+gap+label content width; icon={icon:?} label={label:?} indicator={indicator:?}"
    );
}

#[test]
fn primary_stacked_icon_tabs_use_large_height_and_vertical_content_indicator() {
    let (mut app, window, mut services, mut ui, selected) = tabs_harness();
    let scene = settle_stacked_icon_tabs(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected,
        false,
        TabsVariant::Primary,
    );

    let tab = ui
        .debug_node_visual_bounds(semantics_node(&ui, "m3-tab-a").id)
        .expect("expected selected tab visual bounds");
    let icon = layout_bounds_by_test_id(&ui, &app, window, "m3-tab-a.icon");
    let label = layout_bounds_by_test_id(&ui, &app, window, "m3-tab-a.label");
    let indicator = active_indicator_rect(&scene);
    let content_left = icon.origin.x.0.min(label.origin.x.0);
    let content_right =
        (icon.origin.x.0 + icon.size.width.0).max(label.origin.x.0 + label.size.width.0);
    let expected_indicator_width = (content_right - content_left).max(24.0);
    let icon_center_x = icon.origin.x.0 + icon.size.width.0 * 0.5;
    let label_center_x = label.origin.x.0 + label.size.width.0 * 0.5;

    assert!(
        (tab.size.height.0 - 72.0).abs() <= 0.5,
        "expected stacked icon tab to use the Compose 72px large height, got {tab:?}"
    );
    assert!(
        (icon.size.width.0 - 24.0).abs() <= 0.5 && (icon.size.height.0 - 24.0).abs() <= 0.5,
        "expected stacked tab icon to use the Material 24px icon size, got {icon:?}"
    );
    assert!(
        icon.origin.y.0 < label.origin.y.0 && (icon_center_x - label_center_x).abs() <= 1.0,
        "expected stacked icon to be centered above the label; icon={icon:?} label={label:?}"
    );
    assert!(
        (indicator.origin.x.0 - content_left).abs() <= 1.5
            && (indicator.size.width.0 - expected_indicator_width).abs() <= 2.5,
        "expected primary stacked-icon tab indicator to match the stacked icon/label content width; icon={icon:?} label={label:?} indicator={indicator:?}"
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
