#![cfg(feature = "diagnostics")]

//! Material 3 navigation destination semantics and layout parity tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, Paint, Point, Px, Rect, Scene, SceneOp, SemanticsNode,
    SemanticsOrientation, SemanticsRole, Size, UiServices,
};
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    NavigationBar, NavigationBarItem, NavigationDrawer, NavigationDrawerItem, NavigationRail,
    NavigationRailItem,
};

mod support;

use support::events::key_down;
use support::host::{FakeUiServices, TestHost};
use support::layout::with_padding;
use support::theme::{apply_material_theme, apply_material_theme_rtl};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(360.0)),
    )
}

fn harness() -> (TestHost, AppWindowId, FakeUiServices, UiTree<TestHost>) {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);
    (app, window, services, ui)
}

fn render_navigation_bar(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let bar = NavigationBar::new(selected)
                .a11y_label("Material navigation bar")
                .test_id("m3-navigation-bar")
                .items(vec![
                    NavigationBarItem::new("search", "Search", fret_icons::ids::ui::SEARCH)
                        .test_id("m3-nav-search"),
                    NavigationBarItem::new("settings", "Settings", fret_icons::ids::ui::SETTINGS)
                        .test_id("m3-nav-settings"),
                    NavigationBarItem::new("disabled", "Disabled", fret_icons::ids::ui::SLASH)
                        .disabled(true)
                        .test_id("m3-nav-disabled"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), bar)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn render_navigation_rail(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let rail = NavigationRail::new(selected)
                .a11y_label("Material navigation rail")
                .test_id("m3-navigation-rail")
                .items(vec![
                    NavigationRailItem::new("search", "Search", fret_icons::ids::ui::SEARCH)
                        .test_id("m3-rail-search"),
                    NavigationRailItem::new("play", "Play", fret_icons::ids::ui::PLAY)
                        .test_id("m3-rail-play"),
                    NavigationRailItem::new("disabled", "Disabled", fret_icons::ids::ui::SLASH)
                        .disabled(true)
                        .test_id("m3-rail-disabled"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), rail)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn render_navigation_drawer(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let drawer = NavigationDrawer::new(selected)
                .a11y_label("Material navigation drawer")
                .test_id("m3-navigation-drawer")
                .items(vec![
                    NavigationDrawerItem::new("inbox", "Inbox", fret_icons::ids::ui::SEARCH)
                        .badge_label("99")
                        .test_id("m3-drawer-inbox"),
                    NavigationDrawerItem::new(
                        "settings",
                        "Settings",
                        fret_icons::ids::ui::SETTINGS,
                    )
                    .test_id("m3-drawer-settings"),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), drawer)]
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

fn settle_navigation_bar(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) -> Scene {
    let mut scene = Scene::default();
    for _ in 0..36 {
        render_navigation_bar(ui, app, services, window, selected.clone());
        scene = paint(ui, app, services);
        app.advance_frame();
    }
    scene
}

fn settle_navigation_rail(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: Model<Arc<str>>,
) -> Scene {
    let mut scene = Scene::default();
    for _ in 0..36 {
        render_navigation_rail(ui, app, services, window, selected.clone());
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

fn rect_right(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0
}

fn rect_bottom(rect: Rect) -> f32 {
    rect.origin.y.0 + rect.size.height.0
}

fn rect_center_x(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0 * 0.5
}

fn rect_center_y(rect: Rect) -> f32 {
    rect.origin.y.0 + rect.size.height.0 * 0.5
}

#[derive(Debug, Clone, Copy)]
struct DrawerItemSlotRects {
    chrome: Rect,
    icon: Rect,
    label: Rect,
    badge: Rect,
}

fn drawer_item_slot_rects(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
) -> DrawerItemSlotRects {
    DrawerItemSlotRects {
        chrome: visual_bounds_by_test_id(ui, app, window, "m3-drawer-inbox.chrome"),
        icon: visual_bounds_by_test_id(ui, app, window, "m3-drawer-inbox.icon"),
        label: visual_bounds_by_test_id(ui, app, window, "m3-drawer-inbox.label"),
        badge: visual_bounds_by_test_id(ui, app, window, "m3-drawer-inbox.badge"),
    }
}

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn active_indicator_rect(
    scene: &Scene,
    expected_width: f32,
    expected_height: f32,
    expected_center_x: f32,
    expected_center_y: f32,
) -> Rect {
    let quads: Vec<String> = scene
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
            } if (rect.size.width.0 - expected_width).abs() <= 0.5
                && (rect.size.height.0 - expected_height).abs() <= 0.5
                && matches!(background.paint, Paint::Solid(color) if color.a > 0.0) =>
            {
                Some(rect)
            }
            _ => None,
        })
        .collect();
    candidates.sort_by(|a, b| {
        let da =
            (rect_center_x(*a) - expected_center_x).hypot(rect_center_y(*a) - expected_center_y);
        let db =
            (rect_center_x(*b) - expected_center_x).hypot(rect_center_y(*b) - expected_center_y);
        da.total_cmp(&db)
    });
    candidates.into_iter().next().unwrap_or_else(|| {
        panic!("expected active indicator {expected_width}x{expected_height}; quads={quads:?}")
    })
}

#[test]
fn navigation_bar_exports_horizontal_tablist_semantics_and_collection_metadata() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    render_navigation_bar(&mut ui, &mut app, &mut services, window, selected);

    let list = semantics_node(&ui, "m3-navigation-bar");
    assert_eq!(list.role, SemanticsRole::TabList);
    assert_eq!(
        list.extra.orientation,
        Some(SemanticsOrientation::Horizontal)
    );
    assert_eq!(list.label.as_deref(), Some("Material navigation bar"));

    let search = semantics_node(&ui, "m3-nav-search");
    assert_eq!(search.role, SemanticsRole::Tab);
    assert!(search.flags.selected);
    assert_eq!(search.pos_in_set, Some(1));
    assert_eq!(search.set_size, Some(3));

    let settings = semantics_node(&ui, "m3-nav-settings");
    assert_eq!(settings.role, SemanticsRole::Tab);
    assert!(!settings.flags.selected);
    assert_eq!(settings.pos_in_set, Some(2));
    assert_eq!(settings.set_size, Some(3));

    let disabled = semantics_node(&ui, "m3-nav-disabled");
    assert_eq!(disabled.role, SemanticsRole::Tab);
    assert!(disabled.flags.disabled);
    assert_eq!(disabled.pos_in_set, Some(3));
    assert_eq!(disabled.set_size, Some(3));
}

#[test]
fn navigation_bar_uses_material_item_gap_and_active_indicator_geometry() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    let scene = settle_navigation_bar(&mut ui, &mut app, &mut services, window, selected);

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-navigation-bar.chrome");
    assert_px_close(
        chrome.size.height.0,
        80.0,
        "navigation bar container height",
    );

    let search = visual_bounds_by_test_id(&ui, &app, window, "m3-nav-search.chrome");
    let settings = visual_bounds_by_test_id(&ui, &app, window, "m3-nav-settings.chrome");
    assert_px_close(
        settings.origin.x.0 - rect_right(search),
        8.0,
        "navigation bar item horizontal gap",
    );

    let icon = visual_bounds_by_test_id(&ui, &app, window, "m3-nav-search.icon");
    let indicator =
        active_indicator_rect(&scene, 64.0, 32.0, rect_center_x(icon), rect_center_y(icon));
    assert_px_close(
        rect_center_x(indicator),
        rect_center_x(icon),
        "navigation bar active indicator center x",
    );
    assert_px_close(
        rect_center_y(indicator),
        rect_center_y(icon),
        "navigation bar active indicator center y",
    );
}

#[test]
fn navigation_bar_rtl_arrow_left_moves_to_next_logical_destination() {
    let (mut app, window, mut services, mut ui) = harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    render_navigation_bar(&mut ui, &mut app, &mut services, window, selected.clone());

    let search = semantics_node(&ui, "m3-nav-search").id;
    let settings = semantics_node(&ui, "m3-nav-settings").id;
    ui.set_focus(Some(search));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowLeft));

    assert_eq!(
        ui.focus(),
        Some(settings),
        "expected RTL ArrowLeft to move forward to the next logical navigation destination"
    );
    assert_eq!(
        app.models().get_cloned(&selected).as_deref(),
        Some("settings")
    );
}

#[test]
fn navigation_bar_rtl_theme_direction_mirrors_physical_destination_order() {
    let (mut app, window, mut services, mut ui) = harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    let selected = app.models_mut().insert(Arc::<str>::from("search"));

    render_navigation_bar(&mut ui, &mut app, &mut services, window, selected);

    let search = visual_bounds_by_test_id(&ui, &app, window, "m3-nav-search");
    let settings = visual_bounds_by_test_id(&ui, &app, window, "m3-nav-settings");

    assert!(
        search.origin.x.0 > settings.origin.x.0,
        "expected RTL navigation bar to place the first logical destination to the physical right of the second destination, search={search:?}, settings={settings:?}"
    );
}

#[test]
fn navigation_rail_exports_vertical_tablist_semantics_and_collection_metadata() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("play"));

    render_navigation_rail(&mut ui, &mut app, &mut services, window, selected);

    let list = semantics_node(&ui, "m3-navigation-rail");
    assert_eq!(list.role, SemanticsRole::TabList);
    assert_eq!(list.extra.orientation, Some(SemanticsOrientation::Vertical));
    assert_eq!(list.label.as_deref(), Some("Material navigation rail"));

    let search = semantics_node(&ui, "m3-rail-search");
    assert_eq!(search.role, SemanticsRole::Tab);
    assert!(!search.flags.selected);
    assert_eq!(search.pos_in_set, Some(1));
    assert_eq!(search.set_size, Some(3));

    let play = semantics_node(&ui, "m3-rail-play");
    assert_eq!(play.role, SemanticsRole::Tab);
    assert!(play.flags.selected);
    assert_eq!(play.pos_in_set, Some(2));
    assert_eq!(play.set_size, Some(3));

    let disabled = semantics_node(&ui, "m3-rail-disabled");
    assert_eq!(disabled.role, SemanticsRole::Tab);
    assert!(disabled.flags.disabled);
    assert_eq!(disabled.pos_in_set, Some(3));
    assert_eq!(disabled.set_size, Some(3));
}

#[test]
fn navigation_rail_uses_full_width_56dp_items_and_active_indicator_geometry() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("play"));

    let scene = settle_navigation_rail(&mut ui, &mut app, &mut services, window, selected);

    let chrome = visual_bounds_by_test_id(&ui, &app, window, "m3-navigation-rail.chrome");
    assert_px_close(chrome.size.width.0, 80.0, "navigation rail container width");

    let search = visual_bounds_by_test_id(&ui, &app, window, "m3-rail-search.chrome");
    let play = visual_bounds_by_test_id(&ui, &app, window, "m3-rail-play.chrome");
    assert_px_close(search.size.width.0, 80.0, "navigation rail item width");
    assert_px_close(search.size.height.0, 56.0, "navigation rail item height");
    assert_px_close(
        play.origin.y.0 - rect_bottom(search),
        4.0,
        "navigation rail vertical item gap",
    );

    let icon = visual_bounds_by_test_id(&ui, &app, window, "m3-rail-play.icon");
    let indicator =
        active_indicator_rect(&scene, 56.0, 32.0, rect_center_x(icon), rect_center_y(icon));
    assert_px_close(
        rect_center_x(indicator),
        rect_center_x(icon),
        "navigation rail active indicator center x",
    );
    assert_px_close(
        rect_center_y(indicator),
        rect_center_y(icon),
        "navigation rail active indicator center y",
    );
}

#[test]
fn navigation_drawer_rtl_theme_direction_mirrors_item_slots_and_padding() {
    let (mut app, window, mut services, mut ui) = harness();
    let selected = app.models_mut().insert(Arc::<str>::from("inbox"));
    render_navigation_drawer(&mut ui, &mut app, &mut services, window, selected);
    let ltr = drawer_item_slot_rects(&ui, &app, window);

    assert!(
        ltr.icon.origin.x.0 < ltr.label.origin.x.0 && ltr.label.origin.x.0 < ltr.badge.origin.x.0,
        "expected LTR drawer slots to render icon, label, then badge: {ltr:?}"
    );
    assert_px_close(
        ltr.icon.origin.x.0 - ltr.chrome.origin.x.0,
        16.0,
        "LTR drawer icon uses logical start padding",
    );
    assert_px_close(
        rect_right(ltr.chrome) - rect_right(ltr.badge),
        24.0,
        "LTR drawer badge uses logical end padding",
    );

    let (mut app, window, mut services, mut ui) = harness();
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);
    let selected = app.models_mut().insert(Arc::<str>::from("inbox"));
    render_navigation_drawer(&mut ui, &mut app, &mut services, window, selected);
    let rtl = drawer_item_slot_rects(&ui, &app, window);

    assert!(
        rtl.badge.origin.x.0 < rtl.label.origin.x.0 && rtl.label.origin.x.0 < rtl.icon.origin.x.0,
        "expected RTL drawer slots to render badge, label, then icon: {rtl:?}"
    );
    assert_px_close(
        rtl.badge.origin.x.0 - rtl.chrome.origin.x.0,
        24.0,
        "RTL drawer badge uses logical end padding on the physical left",
    );
    assert_px_close(
        rect_right(rtl.chrome) - rect_right(rtl.icon),
        16.0,
        "RTL drawer icon uses logical start padding on the physical right",
    );
}
