//! Focused Material 3 Menu and DropdownMenu layout, focus, and motion regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, Paint, Point, PointerId, Px, Rect, Scene, SceneOp, SemanticsCheckedState,
    SemanticsNode, SemanticsRole, Size, UiServices,
};
use fret_icons::ids;
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::element::{Length, PressableA11y, PressableProps};
use fret_ui::{UiTree, declarative};
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::menu::{Menu, MenuEntry, MenuGroup, MenuItem, MenuLabel};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};
use fret_ui_material3::{DropdownMenu, DropdownMenuAlign};

mod support;

use support::events::{key_down, key_up, pointer_down, pointer_move};
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::{apply_material_theme, apply_material_theme_rtl};

fn assert_px_close(actual: f32, expected: f32, context: &str) {
    let delta = (actual - expected).abs();
    assert!(
        delta <= 0.5,
        "{context}: expected {expected}px, got {actual}px (delta {delta}px)"
    );
}

fn live_test_id_layout_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_bounds(m.node)
                .or_else(|| ui.debug_node_visual_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live layout bounds for test_id {id}"))
}

fn rect_right(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0
}

fn semantics_node_by_test_id(ui: &UiTree<TestHost>, test_id: &str) -> SemanticsNode {
    ui.semantics_snapshot()
        .and_then(|snapshot| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.test_id.as_deref() == Some(test_id))
                .cloned()
        })
        .unwrap_or_else(|| panic!("expected semantics node for test_id {test_id}"))
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

fn scene_has_intermediate_overlay_motion(scene: &Scene) -> bool {
    let has_alpha = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushOpacity { opacity } if *opacity > 0.01 && *opacity < 0.99
        )
    });
    let has_scale = scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() < 0.001
                    && transform.c.abs() < 0.001
                    && transform.a > 0.8
                    && transform.a < 1.0
                    && transform.d > 0.8
                    && transform.d < 1.0
        )
    });
    has_alpha && has_scale
}

fn paint(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut dyn UiServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    ui.paint_all(app, services, bounds, &mut scene, 1.0);
    scene
}

fn state_layer_alphas_for_chrome(scene: &Scene, chrome: Rect) -> Vec<f32> {
    scene
        .ops()
        .iter()
        .filter_map(|op| match *op {
            SceneOp::Quad {
                rect, background, ..
            } if rect.origin.x.0 >= chrome.origin.x.0 - 0.1
                && rect.origin.y.0 >= chrome.origin.y.0 - 0.1
                && rect.origin.x.0 + rect.size.width.0
                    <= chrome.origin.x.0 + chrome.size.width.0 + 0.1
                && rect.origin.y.0 + rect.size.height.0
                    <= chrome.origin.y.0 + chrome.size.height.0 + 0.1
                && (rect.size.width.0 - chrome.size.width.0).abs() <= 2.1
                && (rect.size.height.0 - chrome.size.height.0).abs() <= 2.1 =>
            {
                match background.paint {
                    Paint::Solid(color) if color.a > 0.0 && color.a < 0.2 => Some(color.a),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

#[test]
fn menu_matches_material_item_geometry_and_semantics() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(320.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let menu = Menu::new()
                    .a11y_label("Material menu")
                    .test_id("m3-menu")
                    .entries(vec![
                        MenuEntry::Item(MenuItem::new("Alpha").test_id("m3-menu-alpha")),
                        MenuEntry::Item(
                            MenuItem::new("Beta disabled")
                                .disabled(true)
                                .test_id("m3-menu-beta-disabled"),
                        ),
                        MenuEntry::Separator,
                        MenuEntry::Item(
                            MenuItem::new(
                                "Gamma with an intentionally long label that exercises max width",
                            )
                            .test_id("m3-menu-gamma"),
                        ),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), menu)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alpha = live_test_id_layout_bounds(&ui, &app, window, "m3-menu-alpha.chrome");
    let beta = live_test_id_layout_bounds(&ui, &app, window, "m3-menu-beta-disabled.chrome");
    let gamma = live_test_id_layout_bounds(&ui, &app, window, "m3-menu-gamma.chrome");

    assert!(
        gamma.size.width.0 >= 111.5 && gamma.size.width.0 <= 280.5,
        "expected menu item width to stay within Material 112..280dp bounds, got {}",
        gamma.size.width.0
    );

    assert_px_close(alpha.size.height.0, 48.0, "alpha menu item height");
    assert_px_close(beta.size.height.0, 48.0, "disabled beta menu item height");
    assert_px_close(gamma.size.height.0, 48.0, "gamma menu item height");
    assert_px_close(alpha.size.width.0, gamma.size.width.0, "alpha item width");
    assert_px_close(
        beta.size.width.0,
        gamma.size.width.0,
        "disabled beta item width",
    );
    assert_px_close(alpha.origin.y.0, 40.0, "menu vertical top padding");
    assert_px_close(
        gamma.origin.y.0 - beta.origin.y.0,
        57.0,
        "separator height plus vertical margins",
    );

    let root = semantics_node_by_test_id(&ui, "m3-menu");
    assert_eq!(root.role, SemanticsRole::Menu);
    assert_eq!(root.label.as_deref(), Some("Material menu"));

    let alpha = semantics_node_by_test_id(&ui, "m3-menu-alpha");
    assert_eq!(alpha.role, SemanticsRole::MenuItem);
    assert!(!alpha.flags.disabled);
    assert_eq!(alpha.pos_in_set, Some(1));
    assert_eq!(alpha.set_size, Some(3));

    let disabled = semantics_node_by_test_id(&ui, "m3-menu-beta-disabled");
    assert_eq!(disabled.role, SemanticsRole::MenuItem);
    assert!(disabled.flags.disabled);
    assert!(!disabled.actions.invoke);
    assert_eq!(disabled.pos_in_set, Some(2));
    assert_eq!(disabled.set_size, Some(3));

    let gamma = semantics_node_by_test_id(&ui, "m3-menu-gamma");
    assert_eq!(gamma.role, SemanticsRole::MenuItem);
    assert_eq!(gamma.pos_in_set, Some(3));
    assert_eq!(gamma.set_size, Some(3));
}

#[test]
fn menu_group_wraps_entries_without_skewing_collection_metadata() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(360.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let menu = Menu::new()
                    .a11y_label("Grouped Material menu")
                    .test_id("m3-grouped-menu")
                    .entries(vec![
                        MenuEntry::Group(
                            MenuGroup::new(vec![
                                MenuEntry::Label(
                                    MenuLabel::new("Edit").test_id("m3-grouped-label-edit"),
                                ),
                                MenuEntry::Item(MenuItem::new("Cut").test_id("m3-grouped-cut")),
                                MenuEntry::Item(MenuItem::new("Copy").test_id("m3-grouped-copy")),
                            ])
                            .a11y_label("Edit actions")
                            .test_id("m3-grouped-edit"),
                        ),
                        MenuEntry::Separator,
                        MenuEntry::Group(
                            MenuGroup::new(vec![MenuEntry::Item(
                                MenuItem::new("Settings").test_id("m3-grouped-settings"),
                            )])
                            .a11y_label("Application actions")
                            .test_id("m3-grouped-application"),
                        ),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(32.0), menu)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let edit_group = semantics_node_by_test_id(&ui, "m3-grouped-edit");
    assert_eq!(edit_group.role, SemanticsRole::Group);
    assert_eq!(edit_group.label.as_deref(), Some("Edit actions"));

    let app_group = semantics_node_by_test_id(&ui, "m3-grouped-application");
    assert_eq!(app_group.role, SemanticsRole::Group);
    assert_eq!(app_group.label.as_deref(), Some("Application actions"));

    let cut = semantics_node_by_test_id(&ui, "m3-grouped-cut");
    assert_eq!(cut.role, SemanticsRole::MenuItem);
    assert_eq!(cut.pos_in_set, Some(1));
    assert_eq!(cut.set_size, Some(3));

    let copy = semantics_node_by_test_id(&ui, "m3-grouped-copy");
    assert_eq!(copy.role, SemanticsRole::MenuItem);
    assert_eq!(copy.pos_in_set, Some(2));
    assert_eq!(copy.set_size, Some(3));

    let settings = semantics_node_by_test_id(&ui, "m3-grouped-settings");
    assert_eq!(settings.role, SemanticsRole::MenuItem);
    assert_eq!(settings.pos_in_set, Some(3));
    assert_eq!(settings.set_size, Some(3));
}

#[test]
fn menu_rich_items_expose_material_slots_and_checked_semantics() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(true);
    let selected = app
        .models_mut()
        .insert(Some(Arc::<str>::from("comfortable")));
    let checked_for_render = checked.clone();
    let selected_for_render = selected.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(360.0)),
    );
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let checked = checked_for_render.clone();
        let selected = selected_for_render.clone();
        declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let menu = Menu::new()
                .a11y_label("Rich Material menu")
                .test_id("m3-rich-menu")
                .entries(vec![
                    MenuEntry::Label(MenuLabel::new("View").test_id("m3-rich-section-view")),
                    MenuEntry::Item(
                        MenuItem::checkbox(checked, "Show toolbar")
                            .supporting_text("Always visible")
                            .shortcut("Ctrl+B")
                            .test_id("m3-rich-toolbar"),
                    ),
                    MenuEntry::Item(
                        MenuItem::radio(selected.clone(), "comfortable", "Comfortable")
                            .trailing_icon(ids::ui::CHEVRON_RIGHT)
                            .test_id("m3-rich-comfortable"),
                    ),
                    MenuEntry::Item(
                        MenuItem::radio(selected, "compact", "Compact").test_id("m3-rich-compact"),
                    ),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(32.0), menu)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let toolbar = live_test_id_layout_bounds(&ui, &app, window, "m3-rich-toolbar.chrome");
    let section = live_test_id_layout_bounds(&ui, &app, window, "m3-rich-section-view");
    assert_px_close(section.size.height.0, 32.0, "menu section label height");
    assert!(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-section-view.text")
            .size
            .width
            .0
            > 0.0
    );
    assert_px_close(toolbar.size.height.0, 64.0, "two-line menu item height");
    assert!(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-toolbar.label")
            .size
            .width
            .0
            > 0.0
    );
    assert!(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-toolbar.supporting-text")
            .size
            .width
            .0
            > 0.0
    );
    assert_px_close(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-toolbar.leading-icon")
            .size
            .width
            .0,
        24.0,
        "checkbox leading indicator slot width",
    );
    assert!(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-toolbar.shortcut")
            .size
            .width
            .0
            > 0.0
    );
    assert_px_close(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-comfortable.trailing-icon")
            .size
            .width
            .0,
        24.0,
        "trailing icon slot width",
    );
    assert_px_close(
        live_test_id_layout_bounds(&ui, &app, window, "m3-rich-compact.leading-icon")
            .size
            .width
            .0,
        24.0,
        "unchecked radio still reserves leading indicator slot",
    );

    let toolbar_sem = semantics_node_by_test_id(&ui, "m3-rich-toolbar");
    assert_eq!(toolbar_sem.role, SemanticsRole::MenuItemCheckbox);
    assert_eq!(toolbar_sem.pos_in_set, Some(1));
    assert_eq!(toolbar_sem.set_size, Some(3));
    assert_eq!(toolbar_sem.flags.checked, Some(true));
    assert_eq!(
        toolbar_sem.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );

    let comfortable_sem = semantics_node_by_test_id(&ui, "m3-rich-comfortable");
    assert_eq!(comfortable_sem.role, SemanticsRole::MenuItemRadio);
    assert_eq!(comfortable_sem.pos_in_set, Some(2));
    assert_eq!(comfortable_sem.set_size, Some(3));
    assert_eq!(comfortable_sem.flags.checked, Some(true));
    assert_eq!(
        comfortable_sem.flags.checked_state,
        Some(SemanticsCheckedState::True)
    );

    let compact_sem = semantics_node_by_test_id(&ui, "m3-rich-compact");
    assert_eq!(compact_sem.role, SemanticsRole::MenuItemRadio);
    assert_eq!(compact_sem.pos_in_set, Some(3));
    assert_eq!(compact_sem.set_size, Some(3));
    assert_eq!(compact_sem.flags.checked, Some(false));
    assert_eq!(
        compact_sem.flags.checked_state,
        Some(SemanticsCheckedState::False)
    );
}

#[test]
fn menu_checkbox_and_radio_items_update_models_on_activation() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let checked = app.models_mut().insert(false);
    let selected = app.models_mut().insert(Some(Arc::<str>::from("list")));
    let checked_for_render = checked.clone();
    let selected_for_render = selected.clone();

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    );
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let checked = checked_for_render.clone();
        let selected = selected_for_render.clone();
        declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let menu = Menu::new()
                .a11y_label("Mutable Material menu")
                .test_id("m3-mutable-menu")
                .entries(vec![
                    MenuEntry::Item(
                        MenuItem::checkbox(checked, "Show toolbar").test_id("m3-mutable-toolbar"),
                    ),
                    MenuEntry::Item(
                        MenuItem::radio(selected, "grid", "Grid").test_id("m3-mutable-grid"),
                    ),
                ])
                .into_element(cx);
            vec![with_padding(cx, Px(24.0), menu)]
        })
    };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let toolbar = semantics_node_id_by_test_id(&ui, "m3-mutable-toolbar")
        .expect("expected toolbar menu item semantics node");
    ui.set_focus(Some(toolbar));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::Enter);
    assert_eq!(app.models().get_copied(&checked), Some(true));

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let grid = semantics_node_id_by_test_id(&ui, "m3-mutable-grid")
        .expect("expected grid menu item semantics node");
    ui.set_focus(Some(grid));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::Enter);
    assert_eq!(
        app.models().get_cloned(&selected).flatten().as_deref(),
        Some("grid")
    );
}

#[test]
fn menu_pressed_state_layer_animates_over_item_chrome() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    );
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let menu = Menu::new()
                    .a11y_label("Material menu")
                    .test_id("m3-motion-menu")
                    .entries(vec![MenuEntry::Item(
                        MenuItem::new("Alpha").test_id("m3-motion-alpha"),
                    )])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), menu)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-motion-alpha.chrome");
    assert!(
        state_layer_alphas_for_chrome(&paint(&mut ui, &mut app, &mut services, bounds), chrome)
            .is_empty(),
        "idle menu item should not paint a visible state layer"
    );

    let press_at = Point::new(
        Px(chrome.origin.x.0 + chrome.size.width.0 * 0.5),
        Px(chrome.origin.y.0 + chrome.size.height.0 * 0.5),
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
        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        animated.extend(state_layer_alphas_for_chrome(
            &paint(&mut ui, &mut app, &mut services, bounds),
            chrome,
        ));
    }

    assert!(
        animated.iter().any(|alpha| *alpha > 0.001 && *alpha < 0.2),
        "expected pressed menu item state layer to animate through partial alpha, got {animated:?}"
    );
}

#[test]
fn menu_roving_focus_includes_disabled_items_without_activation() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    );
    let activated = app.models_mut().insert(false);
    let activated_for_render = activated.clone();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let activated = activated_for_render.clone();
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let disabled_activate = activated.clone();
                let menu = Menu::new()
                    .a11y_label("Material menu")
                    .test_id("m3-focus-menu")
                    .entries(vec![
                        MenuEntry::Item(MenuItem::new("Alpha").test_id("m3-focus-alpha")),
                        MenuEntry::Item(
                            MenuItem::new("Beta disabled")
                                .disabled(true)
                                .on_select(Arc::new(move |host, _cx, _reason| {
                                    let _ = host
                                        .models_mut()
                                        .update(&disabled_activate, |value| *value = true);
                                }))
                                .test_id("m3-focus-beta-disabled"),
                        ),
                        MenuEntry::Item(MenuItem::new("Gamma").test_id("m3-focus-gamma")),
                    ])
                    .into_element(cx);
                vec![with_padding(cx, Px(24.0), menu)]
            })
        };

    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let alpha =
        semantics_node_id_by_test_id(&ui, "m3-focus-alpha").expect("expected alpha semantics node");
    let beta = semantics_node_id_by_test_id(&ui, "m3-focus-beta-disabled")
        .expect("expected disabled beta semantics node");
    let gamma =
        semantics_node_id_by_test_id(&ui, "m3-focus-gamma").expect("expected gamma semantics node");

    ui.set_focus(Some(alpha));
    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert_eq!(
        ui.focus(),
        Some(beta),
        "expected menu ArrowDown to include disabled menu items in roving focus"
    );
    let disabled = semantics_node_by_test_id(&ui, "m3-focus-beta-disabled");
    assert!(disabled.flags.disabled);
    assert!(!disabled.actions.invoke);

    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::Enter);
    assert_eq!(
        app.models().get_copied(&activated),
        Some(false),
        "expected focused disabled menu item not to activate"
    );

    dispatch_key_pair(&mut ui, &mut app, &mut services, KeyCode::ArrowDown);
    let root = render(&mut ui, &mut app, &mut services);
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert_eq!(
        ui.focus(),
        Some(gamma),
        "expected ArrowDown from a disabled menu item to move to the next item"
    );
}

#[test]
fn dropdown_menu_rtl_start_align_uses_material_theme_direction() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(420.0)),
    );
    let open = app.models_mut().insert(true);
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let open = open.clone();
        declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let dropdown = DropdownMenu::new(open)
                .align(DropdownMenuAlign::Start)
                .min_width(Px(260.0))
                .a11y_label("RTL dropdown menu")
                .test_id("m3-dropdown-rtl")
                .into_element(
                    cx,
                    |cx| {
                        let mut props = PressableProps::default();
                        props.layout.size.width = Length::Px(Px(160.0));
                        props.layout.size.height = Length::Px(Px(40.0));
                        props.a11y = PressableA11y {
                            label: Some(Arc::<str>::from("Open RTL menu")),
                            test_id: Some(Arc::<str>::from("m3-dropdown-rtl-trigger")),
                            ..Default::default()
                        };
                        cx.pressable(props, |_cx, _st| Vec::new())
                    },
                    |_cx| {
                        vec![
                            MenuEntry::Item(
                                MenuItem::new("Alpha").test_id("m3-dropdown-rtl-alpha"),
                            ),
                            MenuEntry::Item(MenuItem::new("Beta").test_id("m3-dropdown-rtl-beta")),
                        ]
                    },
                );
            vec![with_padding(cx, Px(120.0), dropdown)]
        })
    };

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected RTL dropdown overlay to open");

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger = live_test_id_layout_bounds(&ui, &app, window, "m3-dropdown-rtl-trigger");
    let menu = live_test_id_layout_bounds(&ui, &app, window, "m3-dropdown-rtl.chrome");

    assert!(
        menu.size.width.0 > trigger.size.width.0 + 40.0,
        "expected test menu to be wider than trigger; trigger={trigger:?}, menu={menu:?}"
    );
    assert_px_close(
        rect_right(menu),
        rect_right(trigger),
        "RTL dropdown start alignment right edge",
    );
    assert!(
        menu.origin.x.0 < trigger.origin.x.0 - 40.0,
        "expected RTL start-aligned wider menu to extend left from trigger; trigger={trigger:?}, menu={menu:?}"
    );
}

#[test]
fn dropdown_menu_matches_material_panel_focus_and_motion() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(680.0), Px(420.0)),
    );
    let open = app.models_mut().insert(false);
    let open_for_render = open.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open_for_render.clone();
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dropdown = DropdownMenu::new(open)
                    .a11y_label("Material dropdown menu")
                    .test_id("m3-dropdown")
                    .into_element(
                        cx,
                        |cx| {
                            let mut props = PressableProps::default();
                            props.layout.size.width = Length::Px(Px(360.0));
                            props.layout.size.height = Length::Px(Px(40.0));
                            props.a11y = PressableA11y {
                                label: Some(Arc::<str>::from("Open menu")),
                                test_id: Some(Arc::<str>::from("m3-dropdown-trigger")),
                                ..Default::default()
                            };
                            cx.pressable(props, |_cx, _st| Vec::new())
                        },
                        |_cx| {
                            vec![
                                MenuEntry::Item(
                                    MenuItem::new("Disabled Alpha")
                                        .disabled(true)
                                        .test_id("m3-dropdown-disabled-alpha"),
                                ),
                                MenuEntry::Item(MenuItem::new("Beta").test_id("m3-dropdown-beta")),
                                MenuEntry::Item(
                                    MenuItem::new("Gamma with an intentionally long menu label")
                                        .test_id("m3-dropdown-gamma"),
                                ),
                            ]
                        },
                    );
                vec![with_padding(cx, Px(32.0), dropdown)]
            })
        };

    run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let _ = app.models_mut().update(&open, |value| *value = true);
    let mut saw_open_motion = false;
    let mut saw_visible_dropdown = false;
    for _ in 0..8 {
        let scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );
        saw_open_motion |= scene_has_intermediate_overlay_motion(&scene);
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        saw_visible_dropdown |= stack.stack.iter().any(|entry| {
            entry.kind == OverlayStackEntryKind::Popover && entry.open && entry.visible
        });
        if saw_open_motion && saw_visible_dropdown {
            break;
        }
    }

    assert!(
        saw_visible_dropdown,
        "expected dropdown popover to become visible"
    );
    assert!(
        saw_open_motion,
        "expected dropdown menu to fade and scale during open"
    );

    let disabled_alpha_bounds =
        live_test_id_layout_bounds(&ui, &app, window, "m3-dropdown-disabled-alpha.chrome");
    let beta_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-dropdown-beta.chrome");
    let gamma_bounds = live_test_id_layout_bounds(&ui, &app, window, "m3-dropdown-gamma.chrome");
    assert!(
        gamma_bounds.size.width.0 >= 111.5 && gamma_bounds.size.width.0 <= 280.5,
        "expected dropdown item width to stay within Material 112..280dp bounds, got {}",
        gamma_bounds.size.width.0
    );
    assert_px_close(
        disabled_alpha_bounds.size.height.0,
        48.0,
        "dropdown first item height",
    );
    assert_px_close(beta_bounds.size.height.0, 48.0, "dropdown beta item height");
    assert_px_close(
        gamma_bounds.size.height.0,
        48.0,
        "dropdown gamma item height",
    );
    assert_px_close(
        disabled_alpha_bounds.origin.y.0,
        84.0,
        "dropdown menu top padding",
    );
    assert_px_close(
        beta_bounds.origin.y.0 - disabled_alpha_bounds.origin.y.0,
        48.0,
        "dropdown consecutive item spacing",
    );

    let disabled_alpha = semantics_node_id_by_test_id(&ui, "m3-dropdown-disabled-alpha")
        .expect("expected disabled alpha semantics node");
    assert_eq!(
        ui.focus(),
        Some(disabled_alpha),
        "expected dropdown initial focus to land on the first menu item, even when disabled"
    );
    let disabled = semantics_node_by_test_id(&ui, "m3-dropdown-disabled-alpha");
    assert!(disabled.flags.disabled);
    assert!(!disabled.actions.invoke);

    let _ = app.models_mut().update(&open, |value| *value = false);
    let close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        true,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_overlay_motion(&close_scene),
        "expected dropdown menu to fade and scale during close"
    );
}

#[test]
fn dropdown_menu_long_content_uses_scrollable_material_viewport() {
    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(260.0)),
    );
    let open = app.models_mut().insert(true);
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let open = open.clone();
            declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dropdown = DropdownMenu::new(open)
                    .a11y_label("Long Material dropdown menu")
                    .test_id("m3-long-dropdown")
                    .max_height(Px(160.0))
                    .into_element(
                        cx,
                        |cx| {
                            let mut props = PressableProps::default();
                            props.layout.size.width = Length::Px(Px(180.0));
                            props.layout.size.height = Length::Px(Px(40.0));
                            props.a11y = PressableA11y {
                                label: Some(Arc::<str>::from("Open long menu")),
                                test_id: Some(Arc::<str>::from("m3-long-dropdown-trigger")),
                                ..Default::default()
                            };
                            cx.pressable(props, |_cx, _st| Vec::new())
                        },
                        |_cx| {
                            (0..24)
                                .map(|idx| {
                                    MenuEntry::Item(
                                        MenuItem::new(format!("Item {idx:02}"))
                                            .test_id(format!("m3-long-dropdown-item-{idx:02}")),
                                    )
                                })
                                .collect()
                        },
                    );
                vec![with_padding(cx, Px(24.0), dropdown)]
            })
        };

    for _ in 0..12 {
        run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    let viewport = live_test_id_layout_bounds(&ui, &app, window, "m3-long-dropdown.viewport");
    assert!(
        viewport.size.height.0 <= 160.5,
        "expected long Material dropdown viewport to clamp to max height, got {viewport:?}"
    );

    let chrome = live_test_id_layout_bounds(&ui, &app, window, "m3-long-dropdown.chrome");
    assert!(
        chrome.size.height.0 > viewport.size.height.0 + 160.0,
        "expected long Material dropdown content to remain scrollable inside the clamped viewport; viewport={viewport:?} chrome={chrome:?}"
    );
}
