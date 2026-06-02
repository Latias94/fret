#![cfg(feature = "diagnostics")]

//! Executable Material minimum touch target conformance tests.

use std::sync::Arc;

use fret_core::{AppWindowId, Point, Px, Rect, Size, UiServices};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui::element::AnyElement;
use fret_ui::elements::ElementContext;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{
    List, ListItem, Menu, MenuEntry, MenuItem, NavigationBar, NavigationBarItem, NavigationDrawer,
    NavigationDrawerItem, NavigationRail, NavigationRailItem, TabItem, Tabs,
};
use serde::Deserialize;

mod support;

use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

const TOUCH_TARGET_FIXTURE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_touch_target_cases_v1.json"
));

#[derive(Debug, Deserialize)]
struct TouchTargetSuite {
    schema_version: u32,
    cases: Vec<TouchTargetCase>,
}

#[derive(Debug, Deserialize)]
struct TouchTargetCase {
    id: String,
    surface: TouchTargetSurface,
    target_test_id: String,
    expected: TouchTargetExpected,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TouchTargetSurface {
    List,
    Menu,
    NavigationBar,
    NavigationDrawer,
    NavigationRail,
    Tabs,
}

#[derive(Debug, Deserialize)]
struct TouchTargetExpected {
    min_width: f32,
    min_height: f32,
}

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(520.0)),
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

fn render_one(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    render: impl FnOnce(&mut ElementContext<'_, TestHost>) -> AnyElement,
) {
    let root =
        fret_ui::declarative::render_root(ui, app, services, window, bounds(), "root", |cx| {
            let child = render(cx);
            vec![with_padding(cx, Px(32.0), child)]
        });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds(), 1.0);
}

fn assert_semantic_touch_target(
    ui: &UiTree<TestHost>,
    case_id: &str,
    test_id: &str,
    min_width: f32,
    min_height: f32,
) {
    let node = semantics_node_id_by_test_id(ui, test_id)
        .unwrap_or_else(|| panic!("{case_id}: expected semantics node for {test_id}"));
    let bounds = ui
        .debug_node_bounds(node)
        .or_else(|| ui.debug_node_visual_bounds(node))
        .unwrap_or_else(|| panic!("{case_id}: expected layout bounds for {test_id}"));

    assert!(
        bounds.size.width.0 >= min_width,
        "{case_id}:{test_id}: expected width >= {min_width}, got {bounds:?}"
    );
    assert!(
        bounds.size.height.0 >= min_height,
        "{case_id}:{test_id}: expected height >= {min_height}, got {bounds:?}"
    );
}

fn load_touch_target_suite() -> TouchTargetSuite {
    serde_json::from_str(TOUCH_TARGET_FIXTURE).expect("touch target fixture JSON must parse")
}

fn run_touch_target_case(case: &TouchTargetCase) {
    let (mut app, window, mut services, mut ui) = harness();

    match case.surface {
        TouchTargetSurface::Tabs => {
            let selected = app.models_mut().insert(Arc::<str>::from("a"));
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                Tabs::new(selected)
                    .a11y_label("Material tabs")
                    .test_id("m3-touch-tabs")
                    .scrollable(true)
                    .items(vec![
                        TabItem::new("a", "Alpha").test_id("m3-touch-tab-alpha"),
                        TabItem::new("b", "Beta").test_id("m3-touch-tab-beta"),
                    ])
                    .into_element(cx)
            });
        }
        TouchTargetSurface::NavigationBar => {
            let selected = app.models_mut().insert(Arc::<str>::from("search"));
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                NavigationBar::new(selected)
                    .a11y_label("Material navigation bar")
                    .test_id("m3-touch-navigation-bar")
                    .items(vec![
                        NavigationBarItem::new("search", "Search", fret_icons::ids::ui::SEARCH)
                            .test_id("m3-touch-nav-bar-search"),
                        NavigationBarItem::new(
                            "settings",
                            "Settings",
                            fret_icons::ids::ui::SETTINGS,
                        )
                        .test_id("m3-touch-nav-bar-settings"),
                    ])
                    .into_element(cx)
            });
        }
        TouchTargetSurface::NavigationRail => {
            let selected = app.models_mut().insert(Arc::<str>::from("search"));
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                NavigationRail::new(selected)
                    .a11y_label("Material navigation rail")
                    .test_id("m3-touch-navigation-rail")
                    .items(vec![
                        NavigationRailItem::new("search", "Search", fret_icons::ids::ui::SEARCH)
                            .test_id("m3-touch-nav-rail-search"),
                        NavigationRailItem::new(
                            "settings",
                            "Settings",
                            fret_icons::ids::ui::SETTINGS,
                        )
                        .test_id("m3-touch-nav-rail-settings"),
                    ])
                    .into_element(cx)
            });
        }
        TouchTargetSurface::NavigationDrawer => {
            let selected = app.models_mut().insert(Arc::<str>::from("inbox"));
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                NavigationDrawer::new(selected)
                    .a11y_label("Material navigation drawer")
                    .test_id("m3-touch-navigation-drawer")
                    .items(vec![
                        NavigationDrawerItem::new("inbox", "Inbox", fret_icons::ids::ui::SEARCH)
                            .test_id("m3-touch-drawer-inbox"),
                        NavigationDrawerItem::new(
                            "settings",
                            "Settings",
                            fret_icons::ids::ui::SETTINGS,
                        )
                        .test_id("m3-touch-drawer-settings"),
                    ])
                    .into_element(cx)
            });
        }
        TouchTargetSurface::Menu => {
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                Menu::new()
                    .a11y_label("Material menu")
                    .test_id("m3-touch-menu")
                    .entries(vec![
                        MenuEntry::Item(MenuItem::new("Alpha").test_id("m3-touch-menu-alpha")),
                        MenuEntry::Item(MenuItem::new("Beta").test_id("m3-touch-menu-beta")),
                    ])
                    .into_element(cx)
            });
        }
        TouchTargetSurface::List => {
            let selected = app.models_mut().insert(Arc::<str>::from("alpha"));
            render_one(&mut ui, &mut app, &mut services, window, |cx| {
                List::new(selected)
                    .a11y_label("Material list")
                    .test_id("m3-touch-list")
                    .items(vec![
                        ListItem::new("alpha", "Alpha").test_id("m3-touch-list-alpha"),
                        ListItem::new("beta", "Beta").test_id("m3-touch-list-beta"),
                    ])
                    .into_element(cx)
            });
        }
    }

    assert_semantic_touch_target(
        &ui,
        &case.id,
        &case.target_test_id,
        case.expected.min_width,
        case.expected.min_height,
    );
}

#[test]
fn navigable_material_rows_enforce_minimum_touch_target_at_runtime() {
    let suite = load_touch_target_suite();
    assert_eq!(suite.schema_version, 1);

    for case in &suite.cases {
        run_touch_target_case(case);
    }
}
