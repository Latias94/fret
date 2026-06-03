#![cfg(feature = "diagnostics")]

//! Material 3 List density, slot, and semantics regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, Point, Px, Rect, SemanticsRole, Size, UiServices,
    semantics::SemanticsNode,
};
use fret_icons::ids;
use fret_runtime::{Model, ModelHost, PlatformCapabilities};
use fret_ui::{UiTree, declarative};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{List, ListItem};

mod support;

use support::events::{key_down, key_up};
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    )
}

fn render_list(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    selected: &'static str,
    items: Vec<ListItem>,
) {
    let model = app.models_mut().insert(Arc::<str>::from(selected));
    let bounds = bounds();
    let root = fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let list = List::new(model.clone())
            .a11y_label("Material list")
            .test_id("m3-list")
            .items(items)
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), list)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn render_list_with_model(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    window: AppWindowId,
    model: Model<Arc<str>>,
    items: Vec<ListItem>,
    loop_navigation: bool,
) {
    let bounds = bounds();
    let root = declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
        let list = List::new(model.clone())
            .a11y_label("Material list")
            .test_id("m3-list")
            .loop_navigation(loop_navigation)
            .items(items)
            .into_element(cx);
        vec![with_padding(cx, Px(32.0), list)]
    });
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
}

fn dispatch_key_pair(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    key: KeyCode,
) {
    ui.dispatch_event(app, services, &key_down(key));
    ui.dispatch_event(app, services, &key_up(key));
}

fn live_test_id_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_visual_bounds(m.node)
                .or_else(|| ui.debug_node_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live bounds for test_id {id}"))
}

fn live_test_id_visual_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| ui.debug_node_visual_bounds(m.node))
        .unwrap_or_else(|| panic!("expected live visual bounds for test_id {id}"))
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

fn focused_test_id(ui: &UiTree<TestHost>) -> Option<String> {
    let focus = ui.focus()?;
    ui.semantics_snapshot().and_then(|snapshot| {
        snapshot
            .nodes
            .iter()
            .find(|node| node.id == focus)
            .and_then(|node| node.test_id.clone())
    })
}

fn selected_value(app: &TestHost, model: &Model<Arc<str>>) -> String {
    app.models()
        .get_cloned(model)
        .expect("selected value model exists")
        .to_string()
}

fn assert_height_close(bounds: Rect, expected: f32, context: &str) {
    let delta = (bounds.size.height.0 - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px height, got {}px",
        bounds.size.height.0
    );
}

fn assert_min_size(bounds: Rect, min_width: f32, min_height: f32, context: &str) {
    assert!(
        bounds.size.width.0 >= min_width && bounds.size.height.0 >= min_height,
        "{context}: expected at least {min_width}x{min_height}px, got {}x{}px",
        bounds.size.width.0,
        bounds.size.height.0
    );
}

fn assert_bounds_close(actual: Rect, expected: Rect, context: &str) {
    let dx = (actual.origin.x.0 - expected.origin.x.0).abs();
    let dy = (actual.origin.y.0 - expected.origin.y.0).abs();
    let dw = (actual.size.width.0 - expected.size.width.0).abs();
    let dh = (actual.size.height.0 - expected.size.height.0).abs();
    assert!(
        dx <= 0.5 && dy <= 0.5 && dw <= 0.5 && dh <= 0.5,
        "{context}: expected bounds {:?}, got {:?}",
        expected,
        actual
    );
}

#[test]
fn list_semantics_expose_collection_selection_and_disabled_state() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_list(
        &mut ui,
        &mut app,
        &mut services,
        window,
        "beta",
        vec![
            ListItem::new("alpha", "Alpha").test_id("m3-list-alpha"),
            ListItem::new("beta", "Beta").test_id("m3-list-beta"),
            ListItem::new("disabled", "Disabled")
                .disabled(true)
                .test_id("m3-list-disabled"),
        ],
    );

    let list = semantics_node(&ui, "m3-list");
    assert_eq!(list.role, SemanticsRole::List);
    assert_eq!(list.label.as_deref(), Some("Material list"));

    let alpha = semantics_node(&ui, "m3-list-alpha");
    assert_eq!(alpha.role, SemanticsRole::ListItem);
    assert_eq!(alpha.label.as_deref(), Some("Alpha"));
    assert!(!alpha.flags.selected);
    assert!(!alpha.flags.disabled);
    assert_eq!(alpha.pos_in_set, Some(1));
    assert_eq!(alpha.set_size, Some(3));

    let beta = semantics_node(&ui, "m3-list-beta");
    assert_eq!(beta.role, SemanticsRole::ListItem);
    assert_eq!(beta.label.as_deref(), Some("Beta"));
    assert!(beta.flags.selected);
    assert!(!beta.flags.disabled);
    assert_eq!(beta.pos_in_set, Some(2));
    assert_eq!(beta.set_size, Some(3));

    let disabled = semantics_node(&ui, "m3-list-disabled");
    assert_eq!(disabled.role, SemanticsRole::ListItem);
    assert!(disabled.flags.disabled);
    assert_eq!(disabled.pos_in_set, Some(3));
    assert_eq!(disabled.set_size, Some(3));
}

#[test]
fn list_two_line_item_uses_material_height_and_slot_part_ids() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_list(
        &mut ui,
        &mut app,
        &mut services,
        window,
        "alpha",
        vec![
            ListItem::new("alpha", "Alpha")
                .supporting_text("Supporting text")
                .trailing_supporting_text("Meta")
                .leading_icon(ids::ui::SEARCH)
                .trailing_icon(ids::ui::CHEVRON_RIGHT)
                .test_id("m3-list-alpha"),
        ],
    );

    assert_height_close(
        live_test_id_bounds(&ui, &app, window, "m3-list-alpha"),
        72.0,
        "two-line list item pressable",
    );
    assert_height_close(
        live_test_id_bounds(&ui, &app, window, "m3-list-alpha.chrome"),
        72.0,
        "two-line list item chrome",
    );
    let item_visual = live_test_id_visual_bounds(&ui, &app, window, "m3-list-alpha");
    let chrome_visual = live_test_id_visual_bounds(&ui, &app, window, "m3-list-alpha.chrome");
    assert_min_size(chrome_visual, 120.0, 40.0, "two-line list item chrome");
    assert_bounds_close(
        item_visual,
        chrome_visual,
        "two-line list item root should match chrome bounds",
    );

    for id in [
        "m3-list-alpha.leading-icon",
        "m3-list-alpha.headline",
        "m3-list-alpha.supporting-text",
        "m3-list-alpha.trailing-supporting-text",
        "m3-list-alpha.trailing-icon",
    ] {
        assert!(
            live_test_id_exists(&ui, &app, window, id),
            "expected stable ListItem part test_id {id}"
        );
    }
}

#[test]
fn list_overline_and_supporting_item_uses_material_three_line_height() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_list(
        &mut ui,
        &mut app,
        &mut services,
        window,
        "alpha",
        vec![
            ListItem::new("alpha", "Alpha")
                .overline_text("Overline")
                .supporting_text("Supporting text")
                .leading_icon(ids::ui::SEARCH)
                .test_id("m3-list-alpha"),
        ],
    );

    assert_height_close(
        live_test_id_bounds(&ui, &app, window, "m3-list-alpha"),
        88.0,
        "three-line list item pressable",
    );
    assert_height_close(
        live_test_id_bounds(&ui, &app, window, "m3-list-alpha.chrome"),
        88.0,
        "three-line list item chrome",
    );
    assert!(live_test_id_exists(
        &ui,
        &app,
        window,
        "m3-list-alpha.overline"
    ));
}

#[test]
fn list_roving_focus_skips_disabled_items_and_updates_selection() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(Arc::<str>::from("alpha"));
    let items = vec![
        ListItem::new("alpha", "Alpha").test_id("m3-list-alpha"),
        ListItem::new("disabled", "Disabled")
            .disabled(true)
            .test_id("m3-list-disabled"),
        ListItem::new("gamma", "Gamma").test_id("m3-list-gamma"),
    ];

    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        true,
    );

    let alpha =
        semantics_node_id_by_test_id(&ui, "m3-list-alpha").expect("expected alpha semantics node");
    ui.set_focus(Some(alpha));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        true,
    );

    assert_eq!(focused_test_id(&ui).as_deref(), Some("m3-list-gamma"));
    assert_eq!(selected_value(&app, &selected), "gamma");

    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::Home);
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        true,
    );

    assert_eq!(focused_test_id(&ui).as_deref(), Some("m3-list-alpha"));
    assert_eq!(selected_value(&app, &selected), "alpha");

    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::End);
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items,
        true,
    );

    assert_eq!(focused_test_id(&ui).as_deref(), Some("m3-list-gamma"));
    assert_eq!(selected_value(&app, &selected), "gamma");
}

#[test]
fn list_roving_respects_loop_navigation_false() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let selected = app.models_mut().insert(Arc::<str>::from("alpha"));
    let items = vec![
        ListItem::new("alpha", "Alpha").test_id("m3-list-alpha"),
        ListItem::new("disabled", "Disabled")
            .disabled(true)
            .test_id("m3-list-disabled"),
        ListItem::new("gamma", "Gamma").test_id("m3-list-gamma"),
    ];

    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        false,
    );

    let alpha =
        semantics_node_id_by_test_id(&ui, "m3-list-alpha").expect("expected alpha semantics node");
    ui.set_focus(Some(alpha));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::ArrowUp);
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        false,
    );

    assert_eq!(
        focused_test_id(&ui).as_deref(),
        Some("m3-list-alpha"),
        "expected ArrowUp at the first enabled item to stay put when loop_navigation=false"
    );
    assert_eq!(selected_value(&app, &selected), "alpha");

    let _ = app
        .models_mut()
        .update(&selected, |value| *value = Arc::<str>::from("gamma"));
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items.clone(),
        false,
    );
    let gamma =
        semantics_node_id_by_test_id(&ui, "m3-list-gamma").expect("expected gamma semantics node");
    ui.set_focus(Some(gamma));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    render_list_with_model(
        &mut ui,
        &mut app,
        &mut services,
        window,
        selected.clone(),
        items,
        false,
    );

    assert_eq!(
        focused_test_id(&ui).as_deref(),
        Some("m3-list-gamma"),
        "expected ArrowDown at the last enabled item to stay put when loop_navigation=false"
    );
    assert_eq!(selected_value(&app, &selected), "gamma");
}

#[test]
fn list_disabled_selected_value_falls_back_to_first_enabled_tab_stop() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    render_list(
        &mut ui,
        &mut app,
        &mut services,
        window,
        "disabled",
        vec![
            ListItem::new("alpha", "Alpha").test_id("m3-list-alpha"),
            ListItem::new("disabled", "Disabled")
                .disabled(true)
                .test_id("m3-list-disabled"),
            ListItem::new("gamma", "Gamma").test_id("m3-list-gamma"),
        ],
    );

    let alpha = semantics_node(&ui, "m3-list-alpha");
    assert!(
        alpha.actions.focus,
        "expected first enabled item to retain the list tab stop when the selected value is disabled"
    );

    let disabled = semantics_node(&ui, "m3-list-disabled");
    assert!(disabled.flags.selected);
    assert!(disabled.flags.disabled);
    assert!(!disabled.actions.focus);
    assert!(!disabled.actions.invoke);
}
