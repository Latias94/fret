//! Focused interaction regression tests for Material 3 Select.

use std::sync::Arc;

use fret_core::{
    AppWindowId, KeyCode, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, Size, UiServices,
};
use fret_runtime::{ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod interaction_harness;
mod support;

use support::events::{key_down, key_up, pointer_down, pointer_up};
use support::goldens::{run_overlay_frame, run_overlay_frame_with_scene_scaled};
use support::host::{FakeUiServices, TestHost};
use support::theme::apply_material_theme;
#[cfg(feature = "diagnostics")]
use support::theme::apply_material_theme_rtl;

#[cfg(feature = "diagnostics")]
fn live_test_id_layout_bounds(
    ui: &UiTree<TestHost>,
    app: &TestHost,
    window: AppWindowId,
    id: &str,
) -> Rect {
    fret_ui::declarative::live_test_id_matches_for_window(app, window, id)
        .into_iter()
        .find_map(|m| {
            ui.debug_node_bounds(m.node)
                .or_else(|| ui.debug_node_visual_bounds(m.node))
        })
        .unwrap_or_else(|| panic!("expected live layout bounds for test_id {id}"))
}

#[cfg(feature = "diagnostics")]
fn horizontal_gaps(child: Rect, parent: Rect) -> (f32, f32) {
    let left = child.origin.x.0 - parent.origin.x.0;
    let right = parent.origin.x.0 + parent.size.width.0 - (child.origin.x.0 + child.size.width.0);
    (left, right)
}

#[cfg(feature = "diagnostics")]
fn rect_right(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0
}

#[cfg(feature = "diagnostics")]
fn rect_center_x(rect: Rect) -> f32 {
    rect.origin.x.0 + rect.size.width.0 * 0.5
}

#[cfg(feature = "diagnostics")]
fn assert_close_px(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.5,
        "expected {label} to be {expected}px, got {actual}px"
    );
}

#[cfg(feature = "diagnostics")]
#[test]
fn select_initial_selected_label_mounts_at_settled_floating_position() {
    use fret_ui_material3::{Select, SelectItem, SelectVariant};

    for (variant, label) in [
        (SelectVariant::Outlined, "outlined"),
        (SelectVariant::Filled, "filled"),
    ] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
        let items: Arc<[SelectItem]> = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ]
        .into();

        let selected_model = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![
                        Select::new(selected_model)
                            .variant(variant)
                            .a11y_label("select")
                            .label("Choice")
                            .placeholder("Pick one")
                            .items(items)
                            .test_id("select-trigger")
                            .into_element(cx),
                    ]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        let first_chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let first_label = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let first_y = first_label.origin.y.0 - first_chrome.origin.y.0;

        for _ in 0..64 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
        }

        let settled_chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let settled_label = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let settled_y = settled_label.origin.y.0 - settled_chrome.origin.y.0;
        let delta = (first_y - settled_y).abs();

        assert!(
            delta <= 0.5,
            "expected {label} initial selected Select label to mount at its settled floating position: first={first_y}, settled={settled_y}, delta={delta}"
        );
    }
}

#[cfg(feature = "diagnostics")]
#[test]
fn select_rtl_label_and_supporting_text_use_logical_inline_insets() {
    use fret_icons::ids;
    use fret_ui_material3::{Select, SelectItem, SelectVariant};

    for (variant, label) in [
        (SelectVariant::Outlined, "outlined"),
        (SelectVariant::Filled, "filled"),
    ] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
        let items: Arc<[SelectItem]> = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ]
        .into();

        let selected_model = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let mut fixed = fret_ui::element::ContainerProps::default();
                    fixed.layout.size.width = fret_ui::element::Length::Px(Px(240.0));
                    fixed.layout.size.height = fret_ui::element::Length::Auto;
                    fixed.layout.overflow = fret_ui::element::Overflow::Visible;

                    vec![cx.container(fixed, move |cx| {
                        vec![
                            Select::new(selected_model)
                                .variant(variant)
                                .a11y_label("select")
                                .label("Choice")
                                .placeholder("Pick one")
                                .supporting_text("Required")
                                .leading_icon(ids::ui::SEARCH)
                                .items(items)
                                .test_id("select-trigger")
                                .into_element(cx),
                        ]
                    })]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        let chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let label_bounds = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let supporting_bounds =
            live_test_id_layout_bounds(&ui, &app, window, "select-trigger.supporting-text");

        let (label_left, label_right) = horizontal_gaps(label_bounds, chrome);
        assert!(
            label_right > label_left + 8.0,
            "expected {label} RTL select label inline-start gap on the right; left={label_left}, right={label_right}"
        );

        let (supporting_left, supporting_right) = horizontal_gaps(supporting_bounds, chrome);
        assert!(
            supporting_right > supporting_left + 8.0,
            "expected {label} RTL select supporting text inline-start gap on the right; left={supporting_left}, right={supporting_right}"
        );
    }
}

#[cfg(feature = "diagnostics")]
#[test]
fn select_rtl_start_aligned_popup_anchors_to_trigger_inline_start() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::select::SelectMenuAlign;
    use fret_ui_material3::{Select, SelectItem};

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

    let selected = app.models_mut().insert(None::<Arc<str>>);
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha"),
        SelectItem::new("beta", "Beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.position = fret_ui::element::PositionStyle::Absolute;
                l.inset = fret_ui::element::InsetStyle {
                    top: Some(Px(80.0)).into(),
                    left: Some(Px(120.0)).into(),
                    right: None.into(),
                    bottom: None.into(),
                };
                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                l.size.height = fret_ui::element::Length::Auto;
                l.overflow = fret_ui::element::Overflow::Visible;

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: l,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            Select::new(selected_model)
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .menu_align(SelectMenuAlign::Start)
                                .match_anchor_width(false)
                                .menu_width_floor(Px(260.0))
                                .items(items)
                                .test_id("select-trigger")
                                .into_element(cx),
                        ]
                    },
                )]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
    let listbox = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.listbox");

    assert!(
        listbox.size.width.0 > trigger.size.width.0 + 40.0,
        "expected the test menu to be wider than the trigger"
    );
    assert_close_px(
        rect_right(listbox),
        rect_right(trigger),
        "RTL start-aligned popup right edge",
    );
    assert!(
        listbox.origin.x.0 < trigger.origin.x.0 - 40.0,
        "expected RTL start-aligned wider popup to extend left from the trigger; trigger={trigger:?}, listbox={listbox:?}"
    );
}

#[cfg(feature = "diagnostics")]
#[test]
fn select_rtl_listbox_items_place_logical_leading_slot_on_right() {
    use fret_icons::ids;
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme_rtl(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(None::<Arc<str>>);
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha")
            .leading_icon(ids::ui::SEARCH)
            .trailing_icon(ids::ui::CHEVRON_RIGHT)
            .test_id("rich-option"),
        SelectItem::new("beta", "Beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut fixed = fret_ui::element::ContainerProps::default();
                fixed.layout.size.width = fret_ui::element::Length::Px(Px(260.0));
                fixed.layout.size.height = fret_ui::element::Length::Auto;
                fixed.layout.overflow = fret_ui::element::Overflow::Visible;

                vec![cx.container(fixed, move |cx| {
                    vec![
                        Select::new(selected_model)
                            .a11y_label("select")
                            .placeholder("Pick one")
                            .items(items)
                            .test_id("select-trigger")
                            .into_element(cx),
                    ]
                })]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let item = live_test_id_layout_bounds(&ui, &app, window, "rich-option.chrome");
    let leading = live_test_id_layout_bounds(&ui, &app, window, "rich-option.leading-icon");
    let trailing = live_test_id_layout_bounds(&ui, &app, window, "rich-option.trailing-icon");
    let item_center = rect_center_x(item);

    assert!(
        rect_center_x(leading) > item_center,
        "expected RTL leading icon on the physical right side; item={item:?}, leading={leading:?}, trailing={trailing:?}"
    );
    assert!(
        rect_center_x(trailing) < item_center,
        "expected RTL trailing icon on the physical left side; item={item:?}, trailing={trailing:?}"
    );
    assert!(
        rect_center_x(leading) > rect_center_x(trailing) + 32.0,
        "expected RTL leading icon to appear to the right of trailing icon; leading={leading:?}, trailing={trailing:?}"
    );
}

#[cfg(feature = "diagnostics")]
#[test]
fn select_focus_floating_label_animates_between_idle_and_focused() {
    use fret_ui_material3::{Select, SelectItem, SelectVariant};

    for (variant, label) in [
        (SelectVariant::Outlined, "outlined"),
        (SelectVariant::Filled, "filled"),
    ] {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let selected = app.models_mut().insert(None::<Arc<str>>);
        let items: Arc<[SelectItem]> = vec![
            SelectItem::new("alpha", "Alpha"),
            SelectItem::new("beta", "Beta"),
        ]
        .into();

        let selected_model = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![
                        Select::new(selected_model)
                            .variant(variant)
                            .a11y_label("select")
                            .label("Choice")
                            .placeholder("Pick one")
                            .items(items)
                            .test_id("select-trigger")
                            .into_element(cx),
                    ]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
                })
            })
            .expect("expected select-trigger in semantics snapshot");
        let idle_chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let idle_label = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let idle_y = idle_label.origin.y.0 - idle_chrome.origin.y.0;

        ui.set_focus(Some(trigger_node));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        let first_chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let first_label = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let first_focus_y = first_label.origin.y.0 - first_chrome.origin.y.0;

        for _ in 0..64 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );
        }

        let settled_chrome = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.chrome");
        let settled_label = live_test_id_layout_bounds(&ui, &app, window, "select-trigger.label");
        let settled_y = settled_label.origin.y.0 - settled_chrome.origin.y.0;

        assert!(
            settled_y < idle_y - 0.5,
            "expected {label} Select focused floating label to settle above idle: idle={idle_y}, settled={settled_y}"
        );
        assert!(
            first_focus_y < idle_y - 0.1,
            "expected {label} Select floating label to start moving on the first focus frame: idle={idle_y}, first={first_focus_y}"
        );
        assert!(
            first_focus_y > settled_y + 0.5,
            "expected {label} Select floating label to animate instead of snapping to the focused endpoint: first={first_focus_y}, settled={settled_y}"
        );
    }
}

fn scene_has_intermediate_rotation(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.b.abs() > 0.01 || transform.c.abs() > 0.01
        )
    })
}

fn scene_has_half_turn_rotation(scene: &Scene) -> bool {
    scene.ops().iter().any(|op| {
        matches!(
            op,
            SceneOp::PushTransform { transform }
                if transform.a < -0.9 && transform.d < -0.9
        )
    })
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
fn select_chevron_rotates_on_first_open_frame() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(None::<Arc<str>>);
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha"),
        SelectItem::new("beta", "Beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected_model)
                        .a11y_label("select")
                        .label("Choice")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected select-trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let first_open_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open),
        "expected Select overlay to open"
    );
    assert!(
        scene_has_intermediate_rotation(&first_open_scene),
        "expected Select chevron to rotate on the first open frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_open_scene),
        "expected Select overlay to fade and scale on the first open frame"
    );

    let mut settled_scene = first_open_scene;
    for _ in 0..64 {
        settled_scene = run_overlay_frame_with_scene_scaled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            1.0,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }
    assert!(
        scene_has_half_turn_rotation(&settled_scene),
        "expected open Select chevron to settle at a half-turn rotation"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let first_close_scene = run_overlay_frame_with_scene_scaled(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        1.0,
        false,
        |ui, app, services| render(ui, app, services),
    );
    assert!(
        scene_has_intermediate_rotation(&first_close_scene),
        "expected Select chevron to rotate on the first close frame"
    );
    assert!(
        scene_has_intermediate_overlay_motion(&first_close_scene),
        "expected Select overlay to fade and scale on the first close frame"
    );
}

#[test]
fn select_dismisses_and_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];

    for (mode, variant, label) in cases {
        let mut app = TestHost::default();
        app.set_global(PlatformCapabilities::default());
        apply_material_theme(&mut app, mode, variant);

        let window = AppWindowId::default();
        let mut services = FakeUiServices;
        let mut ui: UiTree<TestHost> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(560.0), Px(420.0)),
        );

        let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
        let items: Arc<[SelectItem]> = vec![
            SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
            SelectItem::new("beta", "Beta").test_id("select-item-beta"),
            SelectItem::new("charlie", "Charlie (disabled)")
                .disabled(true)
                .test_id("select-item-charlie-disabled"),
        ]
        .into();

        let selected_model = selected.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    vec![
                        Select::new(selected_model)
                            .a11y_label("select")
                            .placeholder("Pick one")
                            .items(items)
                            .test_id("select-trigger")
                            .into_element(cx),
                    ]
                })
            };

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
                })
            })
            .unwrap_or_else(|| panic!("expected select-trigger in semantics snapshot ({label})"));

        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected select-trigger bounds");
        let click_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.set_focus(Some(trigger_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        let mut opened = false;
        for _ in 0..16 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
            {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected select overlay to open on click ({label})");

        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

        let mut closed = false;
        for _ in 0..16 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render(ui, app, services),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
            {
                closed = true;
                break;
            }
        }

        assert!(
            closed,
            "expected select overlay to close on Escape ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected select to restore focus to trigger on Escape ({label})"
        );
    }
}

#[test]
fn select_keyboard_open_sets_initial_focus_and_outside_dismiss_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let cases = [
        (SchemeMode::Dark, DynamicVariant::TonalSpot, "dark/tonal"),
        (SchemeMode::Light, DynamicVariant::TonalSpot, "light/tonal"),
        (
            SchemeMode::Dark,
            DynamicVariant::Expressive,
            "dark/expressive",
        ),
        (
            SchemeMode::Light,
            DynamicVariant::Expressive,
            "light/expressive",
        ),
    ];
    let keyboard_open_keys = [
        (KeyCode::ArrowDown, "arrow_down"),
        (KeyCode::ArrowUp, "arrow_up"),
        (KeyCode::Enter, "enter"),
        (KeyCode::Space, "space"),
    ];

    for (mode, variant, label) in cases {
        for (open_key, key_label) in keyboard_open_keys {
            let mut app = TestHost::default();
            app.set_global(PlatformCapabilities::default());
            apply_material_theme(&mut app, mode, variant);

            let window = AppWindowId::default();
            let mut services = FakeUiServices;
            let mut ui: UiTree<TestHost> = UiTree::new();
            ui.set_window(window);

            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(560.0), Px(420.0)),
            );

            let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
            let underlay_toggled = app.models_mut().insert(false);

            let items: Arc<[SelectItem]> = vec![
                SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
                SelectItem::new("beta", "Beta").test_id("select-item-beta"),
                SelectItem::new("charlie", "Charlie (disabled)")
                    .disabled(true)
                    .test_id("select-item-charlie-disabled"),
            ]
            .into();

            let selected_model = selected.clone();
            let underlay_model = underlay_toggled.clone();
            let render = move |ui: &mut UiTree<TestHost>,
                               app: &mut TestHost,
                               services: &mut dyn UiServices| {
                let selected_model = selected_model.clone();
                let items = items.clone();
                let underlay_model = underlay_model.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let select = Select::new(selected_model)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx);

                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                l
                            },
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(Arc::<str>::from("select-underlay-toggle")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_model);
                            Vec::new()
                        },
                    );

                    let mut props = fret_ui::element::FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                    // Place the underlay above the trigger so the "outside press" point is
                    // guaranteed to be outside the select popover (which opens below the trigger).
                    vec![cx.flex(props, move |_cx| vec![underlay, select])]
                })
            };

            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let trigger_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-trigger in semantics snapshot ({label}, {key_label})")
                });
            let underlay_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-underlay-toggle"))
                            .then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected select-underlay-toggle in semantics snapshot ({label}, {key_label})"
                    )
                });

            ui.set_focus(Some(trigger_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(open_key));
            ui.dispatch_event(&mut app, &mut services, &key_up(open_key));

            let mut opened = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
                {
                    opened = true;
                    break;
                }
            }
            assert!(
                opened,
                "expected select overlay to open on {key_label} ({label})"
            );

            let selected_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-item-beta in semantics snapshot ({label}, {key_label})")
                });
            let mut focused_selected = ui.focus() == Some(selected_option_node);
            for _ in 0..12 {
                if focused_selected {
                    break;
                }
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );
                focused_selected = ui.focus() == Some(selected_option_node);
            }
            if !focused_selected {
                let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
                    ui.focus().and_then(|focused| {
                        snapshot
                            .nodes
                            .iter()
                            .find(|node| node.id == focused)
                            .and_then(|node| node.test_id.as_deref())
                            .map(|s| s.to_string())
                    })
                });
                panic!(
                    "expected Select to move focus to the selected option when opening via keyboard ({label}, {key_label}); focus={:?}, focus_test_id={focused_test_id:?}",
                    ui.focus()
                );
            }

            let underlay_bounds = ui
                .debug_node_visual_bounds(underlay_node)
                .expect("expected underlay bounds");
            let click_at = Point::new(
                Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
                Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
            );
            ui.dispatch_event(
                &mut app,
                &mut services,
                &pointer_down(PointerId(1), click_at),
            );
            ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

            let mut closed = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if !stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
                {
                    closed = true;
                    break;
                }
            }

            assert!(
                closed,
                "expected select overlay to close on outside press after opening via {key_label} ({label})"
            );
            assert_eq!(
                app.models().get_copied(&underlay_toggled),
                Some(false),
                "expected select to prevent underlay activation on outside press ({label}, {key_label})"
            );
            assert_eq!(
                ui.focus(),
                Some(trigger_node),
                "expected select to restore focus to trigger on outside press ({label}, {key_label})"
            );

            ui.set_focus(Some(trigger_node));
            ui.dispatch_event(&mut app, &mut services, &key_down(open_key));
            ui.dispatch_event(&mut app, &mut services, &key_up(open_key));

            let mut reopened = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
                {
                    reopened = true;
                    break;
                }
            }
            assert!(
                reopened,
                "expected select overlay to re-open on {key_label} ({label})"
            );

            let selected_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!("expected select-item-beta in semantics snapshot ({label}, {key_label})")
                });
            let mut focused_selected = ui.focus() == Some(selected_option_node);
            for _ in 0..12 {
                if focused_selected {
                    break;
                }
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    true,
                    |ui, app, services| render(ui, app, services),
                );
                focused_selected = ui.focus() == Some(selected_option_node);
            }
            if !focused_selected {
                let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
                    ui.focus().and_then(|focused| {
                        snapshot
                            .nodes
                            .iter()
                            .find(|node| node.id == focused)
                            .and_then(|node| node.test_id.as_deref())
                            .map(|s| s.to_string())
                    })
                });
                panic!(
                    "expected Select to focus the selected option when reopening via keyboard ({label}, {key_label}); focus={:?}, focus_test_id={focused_test_id:?}",
                    ui.focus()
                );
            }

            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let alpha_option_node: NodeId = ui
                .semantics_snapshot()
                .and_then(|snapshot| {
                    snapshot.nodes.iter().find_map(|node| {
                        (node.test_id.as_deref() == Some("select-item-alpha")).then_some(node.id)
                    })
                })
                .unwrap_or_else(|| {
                    panic!(
                        "expected select-item-alpha in semantics snapshot ({label}, {key_label})"
                    )
                });
            assert_eq!(
                ui.focus(),
                Some(alpha_option_node),
                "expected ArrowDown to rove focus to the next enabled option (wrap + skip disabled) ({label}, {key_label})"
            );

            ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Enter));
            ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Enter));

            let mut closed_after_select = false;
            for _ in 0..24 {
                run_overlay_frame(
                    &mut ui,
                    &mut app,
                    &mut services,
                    window,
                    bounds,
                    false,
                    |ui, app, services| render(ui, app, services),
                );

                let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
                if !stack
                    .stack
                    .iter()
                    .any(|e| e.kind == OverlayStackEntryKind::Popover && e.visible)
                {
                    closed_after_select = true;
                    break;
                }
            }

            assert!(
                closed_after_select,
                "expected select overlay to close after selecting an option ({label}, {key_label})"
            );
            assert_eq!(
                ui.focus(),
                Some(trigger_node),
                "expected select to restore focus to trigger after selecting an option ({label}, {key_label})"
            );
            assert_eq!(
                app.models().get_cloned(&selected),
                Some(Some(Arc::<str>::from("alpha"))),
                "expected Enter to select the focused option ({label}, {key_label})"
            );
        }
    }
}

#[test]
fn select_roving_scrolls_focused_option_into_view() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("item-0")));
    let mut items_vec: Vec<SelectItem> = Vec::new();
    for i in 0..20 {
        let value: Arc<str> = Arc::from(format!("item-{i}"));
        let label: Arc<str> = Arc::from(format!("Item {i}"));
        items_vec.push(
            SelectItem::new(value.clone(), label).test_id(Arc::from(format!("select-item-{i}"))),
        );
    }
    let items: Arc<[SelectItem]> = items_vec.into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected = selected.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected select-trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    for _ in 0..12 {
        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let listbox_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger.listbox")).then_some(node.id)
            })
        })
        .expect("expected select-trigger.listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let focused = ui.focus().expect("expected focused node after roving");
    let focused_bounds = ui
        .debug_node_visual_bounds(focused)
        .expect("expected focused bounds");

    let epsilon = 0.01;
    let listbox_top = listbox_bounds.origin.y.0;
    let listbox_bottom = listbox_bounds.origin.y.0 + listbox_bounds.size.height.0;
    let focused_top = focused_bounds.origin.y.0;
    let focused_bottom = focused_bounds.origin.y.0 + focused_bounds.size.height.0;
    assert!(
        focused_top + epsilon >= listbox_top && focused_bottom - epsilon <= listbox_bottom,
        "expected focused option to be visible within listbox viewport after roving"
    );
}

#[test]
fn select_open_scrolls_selected_option_into_view() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("item-18")));
    let mut items_vec: Vec<SelectItem> = Vec::new();
    for i in 0..30 {
        let value: Arc<str> = Arc::from(format!("item-{i}"));
        let label: Arc<str> = Arc::from(format!("Item {i}"));
        items_vec.push(
            SelectItem::new(value.clone(), label).test_id(Arc::from(format!("select-item-{i}"))),
        );
    }
    let items: Arc<[SelectItem]> = items_vec.into();

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected = selected.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let listbox_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger.listbox")).then_some(node.id)
            })
        })
        .expect("expected select-trigger.listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let selected_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-18")).then_some(node.id)
            })
        })
        .expect("expected select-item-18 in semantics snapshot");
    let selected_bounds = ui
        .debug_node_visual_bounds(selected_node)
        .expect("expected selected option bounds");

    let epsilon = 0.01;
    let listbox_top = listbox_bounds.origin.y.0;
    let listbox_bottom = listbox_bounds.origin.y.0 + listbox_bounds.size.height.0;
    let selected_top = selected_bounds.origin.y.0;
    let selected_bottom = selected_bounds.origin.y.0 + selected_bounds.size.height.0;
    assert!(
        selected_top + epsilon >= listbox_top && selected_bottom - epsilon <= listbox_bottom,
        "expected the selected option to be visible within listbox viewport on open"
    );
}

#[test]
fn select_menu_matches_anchor_width_and_clamps_height_to_available_space() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("v0")));
    let items: Arc<[SelectItem]> = (0..40)
        .map(|i| SelectItem::new(Arc::<str>::from(format!("v{i}")), format!("Item {i}")))
        .collect::<Vec<_>>()
        .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let mut l = fret_ui::element::LayoutStyle::default();
                l.position = fret_ui::element::PositionStyle::Absolute;
                l.inset = fret_ui::element::InsetStyle {
                    top: Some(Px(200.0)).into(),
                    left: Some(Px(24.0)).into(),
                    right: None.into(),
                    bottom: None.into(),
                };
                l.size.width = fret_ui::element::Length::Px(Px(240.0));
                l.size.height = fret_ui::element::Length::Auto;
                l.overflow = fret_ui::element::Overflow::Visible;

                vec![cx.container(
                    fret_ui::element::ContainerProps {
                        layout: l,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![
                            Select::new(selected_model)
                                .a11y_label("select")
                                .placeholder("Pick one")
                                .items(items)
                                .test_id("select-trigger")
                                .into_element(cx),
                        ]
                    },
                )]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected select-trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    for _ in 0..20 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
    }

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot");

    let listbox_node: NodeId = snapshot
        .nodes
        .iter()
        .find_map(|node| {
            (node.test_id.as_deref() == Some("select-trigger.listbox")).then_some(node.id)
        })
        .expect("expected select-trigger.listbox in semantics snapshot");
    let listbox_bounds = ui
        .debug_node_visual_bounds(listbox_node)
        .expect("expected listbox bounds");

    let epsilon = 0.01;
    assert!(
        (listbox_bounds.size.width.0 - trigger_bounds.size.width.0).abs() <= epsilon,
        "expected listbox width to match trigger width"
    );

    let collision_top = 48.0;
    let collision_bottom = 48.0;
    let gap = 4.0;

    let outer_top = bounds.origin.y.0 + collision_top;
    let outer_bottom = bounds.origin.y.0 + bounds.size.height.0 - collision_bottom;
    let anchor_top = trigger_bounds.origin.y.0;
    let anchor_bottom = trigger_bounds.origin.y.0 + trigger_bounds.size.height.0;

    let available_above = anchor_top - (outer_top + gap);
    let available_below = outer_bottom - (anchor_bottom + gap);
    let available = available_above.max(available_below).max(0.0);

    assert!(
        listbox_bounds.size.height.0 <= available + epsilon,
        "expected listbox height to clamp to available space (got {}, want <= {})",
        listbox_bounds.size.height.0,
        available
    );
    assert!(
        (listbox_bounds.size.height.0 - available).abs() <= 0.5,
        "expected listbox height to match available space when content overflows (got {}, want ~ {})",
        listbox_bounds.size.height.0,
        available
    );
}

#[test]
fn select_exposes_combobox_controls_and_listbox_labelled_by_relations() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha"),
        SelectItem::new("beta", "Beta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected_model)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected select-trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    let mut opened = false;
    for _ in 0..16 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open on click");

    // One extra frame: the trigger's `controls_element` is resolved via last-frame element IDs.
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        |ui, app, services| render(ui, app, services),
    );
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let trigger = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-trigger"))
        .expect("select trigger semantics node");
    assert!(
        trigger.flags.expanded,
        "select trigger should report expanded=true while open"
    );

    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("select-trigger.listbox"))
        .expect("select listbox semantics node");

    assert!(
        trigger.controls.contains(&listbox.id),
        "select trigger should control the listbox"
    );
    assert!(
        listbox.labelled_by.contains(&trigger.id),
        "select listbox should be labelled by the trigger"
    );
}

#[test]
fn select_listbox_typeahead_moves_focus_skipping_disabled_options() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha").test_id("select-item-alpha"),
        SelectItem::new("beta", "Beta").test_id("select-item-beta"),
        SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("select-item-charlie-disabled"),
        SelectItem::new("delta", "Delta").test_id("select-item-delta"),
    ]
    .into();

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected_model)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    let beta_option_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
            })
        })
        .expect("expected select-item-beta in semantics snapshot");
    assert_eq!(
        ui.focus(),
        Some(beta_option_node),
        "expected select to focus the selected option when opening via keyboard"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyC));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyC));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );
    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-beta"),
        "expected typeahead to ignore disabled matches (KeyC)"
    );

    // Wait for the typeahead buffer to expire (select installs a prefix-buffer typeahead policy).
    for _ in 0..40 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyD));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyD));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead to rove focus to the matching option (KeyD)"
    );
}

#[test]
fn select_typeahead_delay_controls_buffer_expiration() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(560.0), Px(420.0)),
    );

    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("beta", "Beta").test_id("select-item-beta"),
        SelectItem::new("delta", "Delta").test_id("select-item-delta"),
        SelectItem::new("echo", "Echo").test_id("select-item-echo"),
    ]
    .into();

    let delay_ms = 1000;
    let timeout_ticks = fret_ui_material3::motion::ms_to_frames(delay_ms);

    let selected_model = selected.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let selected_model = selected_model.clone();
            let items = items.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![
                    Select::new(selected_model)
                        .a11y_label("select")
                        .placeholder("Pick one")
                        .items(items)
                        .typeahead_delay_ms(delay_ms)
                        .test_id("select-trigger")
                        .into_element(cx),
                ]
            })
        };

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let trigger_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-trigger")).then_some(node.id)
            })
        })
        .expect("expected select-trigger in semantics snapshot");

    ui.set_focus(Some(trigger_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut opened = false;
    for _ in 0..24 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        if stack
            .stack
            .iter()
            .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected select overlay to open");

    let beta_option_node: NodeId = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find_map(|node| {
                (node.test_id.as_deref() == Some("select-item-beta")).then_some(node.id)
            })
        })
        .expect("expected select-item-beta in semantics snapshot");
    assert_eq!(
        ui.focus(),
        Some(beta_option_node),
        "expected select to focus the selected option when opening via keyboard"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyD));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyD));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead (KeyD) to focus Delta"
    );

    // The buffer should still be active: `d` + `e` => "de" matches Delta, not Echo.
    for _ in 0..timeout_ticks.saturating_sub(1) {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyE));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyE));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-delta"),
        "expected typeahead buffer to keep 'de' and stay on Delta before timeout"
    );

    // Now let the buffer expire, then 'e' should match Echo.
    for _ in 0..(timeout_ticks + 2) {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
    }

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::KeyE));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::KeyE));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.as_deref())
        })
    });
    assert_eq!(
        focused_test_id,
        Some("select-item-echo"),
        "expected typeahead buffer to expire and 'e' to match Echo after timeout"
    );
}
