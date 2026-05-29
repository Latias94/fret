//! Focused Material 3 Menu and DropdownMenu layout, focus, and motion regression tests.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, Point, Px, Rect, Scene, SceneOp, SemanticsNode, SemanticsRole, Size,
    UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::element::{Length, PressableA11y, PressableProps};
use fret_ui::{UiTree, declarative};
use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
use fret_ui_material3::DropdownMenu;
use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{key_down, key_up};
use support::goldens::run_overlay_frame_with_scene_scaled;
use support::host::{FakeUiServices, TestHost};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

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
