use std::{collections::HashMap, sync::Arc};

use fret_core::{
    AppWindowId, Event, KeyCode, NodeId, Point, PointerId, Px, Rect, Scene, SceneOp, Size,
    UiServices,
};
use fret_runtime::{Effect, ModelHost, PlatformCapabilities};
use fret_ui::UiTree;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_material3::tokens::v30::{DynamicVariant, SchemeMode};

mod support;

use support::events::{
    key_down, key_up, pointer_down, pointer_move, pointer_move_touch, pointer_up,
};
use support::goldens::run_overlay_frame;
use support::host::{FakeUiServices, TestHost};
use support::interaction_harness::{
    QuadGeomSig, SceneSig, scene_quad_geometry_signature, scene_signature,
};
use support::layout::{semantics_node_id_by_test_id, with_padding};
use support::theme::apply_material_theme;

// Snackbar, Menu, Dialog, Tooltip, and dropdown interaction regressions.

#[test]
fn snackbar_action_emits_command_and_dismisses() {
    use fret_runtime::CommandId;
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    let cmd = CommandId::new("material3_snackbar_action");

    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(
            &mut action_host,
            window,
            Snackbar::new("Saved").action_id("Undo", cmd.clone()),
        );
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let action_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("Undo") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected the toast action text (Undo) to appear in semantics");

    let mut action_button_id = action_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&action_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        action_button_id = parent;
    }

    let action_bounds = ui
        .debug_node_visual_bounds(action_button_id)
        .expect("expected toast action bounds");
    let click_at = Point::new(
        Px(action_bounds.origin.x.0 + action_bounds.size.width.0 * 0.5),
        Px(action_bounds.origin.y.0 + action_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        app.effects.iter().any(|effect| matches!(
            effect,
            Effect::Command { command, .. } if *command == cmd
        )),
        "expected clicking snackbar action to emit a command effect"
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
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

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
}

#[test]
fn snackbar_dismiss_button_dismisses_without_emitting_command() {
    use fret_ui::action::UiActionHostAdapter;
    use fret_ui_kit::ToastStore;
    use fret_ui_material3::{Snackbar, SnackbarController, SnackbarHost};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(860.0), Px(520.0)),
    );

    let store = app.models_mut().insert(ToastStore::default());
    let controller = SnackbarController::new(store.clone());
    {
        let mut action_host = UiActionHostAdapter { app: &mut app };
        let _id = controller.show(&mut action_host, window, Snackbar::new("Dismiss me"));
    }

    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let store = store.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                vec![SnackbarHost::new(store).max_snackbars(1).into_element(cx)]
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

    let toast_root = ui
        .semantics_snapshot()
        .and_then(|snapshot| {
            snapshot.nodes.iter().find(|node| {
                node.test_id
                    .as_deref()
                    .is_some_and(|id| id.starts_with("toast-entry-"))
            })
        })
        .expect("expected a toast-entry semantics node");

    let toast_root_id = toast_root.id;

    let snapshot = ui
        .semantics_snapshot()
        .expect("expected semantics snapshot for toast");
    let by_id: HashMap<NodeId, &fret_core::SemanticsNode> =
        snapshot.nodes.iter().map(|n| (n.id, n)).collect();

    let is_descendant_of = |mut node: NodeId, ancestor: NodeId| -> bool {
        let mut guard = 0usize;
        while guard < 256 {
            if node == ancestor {
                return true;
            }
            guard += 1;
            let Some(parent) = by_id.get(&node).and_then(|n| n.parent) else {
                return false;
            };
            node = parent;
        }
        false
    };

    let close_text = snapshot
        .nodes
        .iter()
        .find(|node| {
            node.label.as_deref() == Some("\u{00D7}") && is_descendant_of(node.id, toast_root_id)
        })
        .expect("expected toast close glyph (×) to appear in semantics");

    let mut close_button_id = close_text.id;
    let mut guard = 0usize;
    while guard < 256 {
        guard += 1;
        let Some(node) = by_id.get(&close_button_id) else {
            break;
        };
        if node.role == fret_core::SemanticsRole::Button {
            break;
        }
        let Some(parent) = node.parent else {
            break;
        };
        close_button_id = parent;
    }

    let close_bounds = ui
        .debug_node_visual_bounds(close_button_id)
        .expect("expected close button bounds");
    let click_at = Point::new(
        Px(close_bounds.origin.x.0 + close_bounds.size.width.0 * 0.5),
        Px(close_bounds.origin.y.0 + close_bounds.size.height.0 * 0.5),
    );

    app.effects.clear();
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(1), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

    assert!(
        !app.effects
            .iter()
            .any(|effect| matches!(effect, Effect::Command { .. })),
        "expected clicking snackbar dismiss button not to emit a command effect",
    );

    let remove_tokens: Vec<fret_runtime::TimerToken> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                window: Some(w),
                token,
                after: _,
                repeat: None,
            } if *w == window => Some(*token),
            _ => None,
        })
        .collect();
    assert!(
        !remove_tokens.is_empty(),
        "expected snackbar dismiss to schedule a timer for removal"
    );

    for token in remove_tokens {
        ui.dispatch_event(&mut app, &mut services, &Event::Timer { token });
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

    let has_toast = ui.semantics_snapshot().is_some_and(|snapshot| {
        snapshot.nodes.iter().any(|node| {
            node.test_id
                .as_deref()
                .is_some_and(|id| id.starts_with("toast-entry-"))
        })
    });
    assert!(
        !has_toast,
        "expected snackbar to be removed after dismiss timer fires"
    );
}

#[test]
fn menu_pressed_scene_structure_is_stable() {
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem};

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
            Size::new(Px(360.0), Px(260.0)),
        );

        let entries = vec![
            MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
            MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
            MenuEntry::Item(MenuItem::new("C").test_id("menu-item-c")),
        ];

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = Menu::new()
                        .entries(entries.clone())
                        .a11y_label("menu")
                        .test_id("menu")
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), menu)]
                })
            };

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let item_b: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("menu-item-b") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .expect("expected menu-item-b in semantics snapshot");
        let item_bounds = ui
            .debug_node_visual_bounds(item_b)
            .expect("expected menu-item-b visual bounds");
        let press_at = Point::new(
            Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
            Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), press_at),
        );

        let mut baseline_structure: Option<Vec<SceneSig>> = None;
        let mut baseline_quads: Option<Vec<QuadGeomSig>> = None;
        for frame in 0..24 {
            app.advance_frame();
            let root = render(&mut ui, &mut app, &mut services);
            ui.set_root(root);
            ui.layout_all(&mut app, &mut services, bounds, 1.0);

            let mut scene = Scene::default();
            ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

            if (2..7).contains(&frame) {
                let sig = scene_signature(&scene);
                if let Some(prev) = baseline_structure.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep a stable scene structure while pressed ({label})"
                    );
                } else {
                    baseline_structure = Some(sig);
                }
            }

            if frame >= 16 {
                let sig = scene_quad_geometry_signature(&scene);
                if let Some(prev) = baseline_quads.as_ref() {
                    assert_eq!(
                        sig, *prev,
                        "expected Menu to keep stable quad geometry after animations settle ({label})"
                    );
                } else {
                    baseline_quads = Some(sig);
                }
            }
        }

        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), press_at));
    }
}

#[test]
fn menu_style_overrides_apply_to_container_and_label() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::menu::{Menu, MenuEntry, MenuItem, MenuStyle};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(260.0)),
    );

    let override_bg = Color {
        r: 0.9,
        g: 0.1,
        b: 0.2,
        a: 1.0,
    };
    let override_label = Color {
        r: 0.1,
        g: 0.8,
        b: 0.3,
        a: 1.0,
    };

    let style = MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_label,
        ))));

    let entries = vec![
        MenuEntry::Item(MenuItem::new("A").test_id("menu-item-a")),
        MenuEntry::Item(MenuItem::new("B").test_id("menu-item-b")),
    ];

    let root = fret_ui::declarative::render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "root",
        |cx| {
            let menu = Menu::new()
                .entries(entries.clone())
                .a11y_label("menu")
                .test_id("menu")
                .style(style.clone())
                .into_element(cx);
            vec![with_padding(cx, Px(24.0), menu)]
        },
    );
    ui.set_root(root);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let mut scene = Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Quad { background, .. } if background.paint == fret_core::Paint::Solid(override_bg)
            )
        }),
        "expected MenuStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| {
                matches!(
                    op,
                    SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_label)
                )
            }),
        "expected MenuStyle.item_label_color to affect at least one text draw op"
    );
}

#[test]
fn dialog_focus_is_contained_and_restored_across_schemes() {
    use fret_ui_material3::{Button, Dialog, DialogAction};

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
            Size::new(Px(420.0), Px(320.0)),
        );

        let open = app.models_mut().insert(false);

        let render =
            |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let dialog = Dialog::new(open.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![dialog]
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
                assert_eq!(snapshot.barrier_root, None);
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("dialog-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dialog-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));
        assert_eq!(ui.focus(), Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        assert!(
            snapshot.barrier_root.is_some(),
            "expected modal barrier root while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("dialog.scrim")),
            "expected dialog scrim node while dialog is open ({label})"
        );
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to move into dialog layer while open ({label})"
        );

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
        assert_ne!(
            ui.focus(),
            Some(trigger_node),
            "expected modal barrier to enforce focus containment ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = false);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected focus to restore to trigger on close ({label})"
        );

        let mut saw_barrier_cleared = false;
        for _ in 0..40 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                true,
                |ui, app, services| render(ui, app, services),
            );

            let snapshot = ui
                .semantics_snapshot()
                .expect("expected semantics snapshot");
            if snapshot.barrier_root.is_none() {
                saw_barrier_cleared = true;
                break;
            }
        }
        assert!(
            saw_barrier_cleared,
            "expected dialog barrier to unmount after close transition ({label})"
        );
    }
}

#[test]
fn dialog_style_overrides_apply_to_container_and_text() {
    use fret_core::Color;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};
    use fret_ui_material3::{Button, Dialog, DialogAction, DialogStyle};

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

    let open = app.models_mut().insert(true);

    let override_bg = Color {
        r: 0.2,
        g: 0.2,
        b: 0.9,
        a: 1.0,
    };
    let override_headline = Color {
        r: 0.9,
        g: 0.9,
        b: 0.2,
        a: 1.0,
    };
    let override_supporting = Color {
        r: 0.8,
        g: 0.2,
        b: 0.8,
        a: 1.0,
    };

    let style = DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(override_bg))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_headline,
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            override_supporting,
        ))));

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let dialog = Dialog::new(open.clone())
                .headline("Dialog")
                .supporting_text("Body")
                .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                .style(style.clone())
                .test_id("dialog")
                .into_element(
                    cx,
                    |cx| {
                        let trigger = Button::new("Underlay focus probe")
                            .test_id("dialog-trigger")
                            .into_element(cx);
                        with_padding(cx, Px(24.0), trigger)
                    },
                    |_cx| Vec::new(),
                );
            vec![dialog]
        })
    };

    let mut scene = None;
    for _ in 0..3 {
        use fret_ui_kit::OverlayController;

        app.advance_frame();
        OverlayController::begin_frame(&mut app, window);

        let root = render(&mut ui, &mut app, &mut services);
        ui.set_root(root);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);

        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut next = Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut next, 1.0);
        scene = Some(next);
    }

    let scene = scene.expect("expected rendered scene");

    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Quad { background, .. } if background.paint == fret_core::Paint::Solid(override_bg)
            )
        }),
        "expected DialogStyle.container_background to affect at least one quad background"
    );
    assert!(
        scene
            .ops()
            .iter()
            .any(|op| {
                matches!(
                    op,
                    SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_headline)
                )
            }),
        "expected DialogStyle.headline_color to affect at least one text draw op"
    );
    assert!(
        scene.ops().iter().any(|op| {
            matches!(
                op,
                SceneOp::Text { paint, .. } if paint.paint == fret_core::Paint::Solid(override_supporting)
            )
        }),
        "expected DialogStyle.supporting_text_color to affect at least one text draw op"
    );
}

#[test]
fn select_inside_dialog_closes_inner_popover_before_modal_dialog() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, Dialog, DialogAction, Select, SelectItem};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(480.0)),
    );

    let dialog_open = app.models_mut().insert(false);
    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha").test_id("dialog-select-alpha"),
        SelectItem::new("beta", "Beta").test_id("dialog-select-beta"),
        SelectItem::new("gamma", "Gamma").test_id("dialog-select-gamma"),
    ]
    .into();

    let dialog_open_for_render = dialog_open.clone();
    let selected_for_render = selected.clone();
    let items_for_render = items.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let dialog_open = dialog_open_for_render.clone();
            let selected = selected_for_render.clone();
            let items = items_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = Dialog::new(dialog_open.clone())
                    .headline("Preferences")
                    .supporting_text("The nested Select should behave like the top overlay.")
                    .actions(vec![DialogAction::new("Done").test_id("dialog-done")])
                    .test_id("dialog")
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = Button::new("Open dialog")
                                .test_id("dialog-trigger")
                                .into_element(cx);
                            with_padding(cx, Px(24.0), trigger)
                        },
                        move |cx| {
                            vec![
                                Select::new(selected.clone())
                                    .a11y_label("Dialog Select")
                                    .label("Project")
                                    .placeholder("Pick one")
                                    .items(items.clone())
                                    .test_id("dialog-select")
                                    .into_element(cx),
                            ]
                        },
                    );
                vec![dialog]
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

    let trigger_node = semantics_node_id_by_test_id(&ui, "dialog-trigger")
        .expect("expected dialog trigger before opening");
    ui.set_focus(Some(trigger_node));
    let _ = app.models_mut().update(&dialog_open, |open| *open = true);

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let select_node =
        semantics_node_id_by_test_id(&ui, "dialog-select").expect("expected select inside dialog");
    ui.set_focus(Some(select_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut saw_nested_popover = false;
    for _ in 0..32 {
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
        let visible_open: Vec<_> = stack
            .stack
            .iter()
            .filter(|entry| entry.open && entry.visible)
            .map(|entry| entry.kind)
            .collect();

        if visible_open.contains(&OverlayStackEntryKind::Modal)
            && visible_open.contains(&OverlayStackEntryKind::Popover)
        {
            assert_eq!(
                visible_open.last().copied(),
                Some(OverlayStackEntryKind::Popover),
                "expected the nested Select popover to be above the dialog modal layer"
            );
            saw_nested_popover = true;
            break;
        }
    }
    assert!(saw_nested_popover, "expected Select popover above Dialog");

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut popover_closed = false;
    for _ in 0..32 {
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
        popover_closed = !stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.visible);
        if popover_closed {
            break;
        }
    }

    assert!(
        popover_closed,
        "expected first Escape to close the Select popover"
    );
    assert_eq!(
        app.models().get_copied(&dialog_open),
        Some(true),
        "expected first Escape to leave the Dialog open"
    );
    let select_node_after_close = semantics_node_id_by_test_id(&ui, "dialog-select")
        .expect("expected select trigger after closing nested popover");
    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.clone())
        })
    });
    assert_eq!(
        ui.focus(),
        Some(select_node_after_close),
        "expected focus to restore to the Select trigger inside the Dialog; focused_test_id={focused_test_id:?}"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models().get_copied(&dialog_open),
        Some(false),
        "expected second Escape to close the Dialog"
    );
}

#[test]
fn autocomplete_inside_dialog_escape_closes_inner_popover_before_modal_dialog() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Autocomplete, AutocompleteItem, Button, Dialog, DialogAction};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(640.0), Px(480.0)),
    );

    let dialog_open = app.models_mut().insert(false);
    let query = app.models_mut().insert(String::new());
    let selected = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]
    .into();

    let dialog_open_for_render = dialog_open.clone();
    let query_for_render = query.clone();
    let selected_for_render = selected.clone();
    let items_for_render = items.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let dialog_open = dialog_open_for_render.clone();
            let query = query_for_render.clone();
            let selected = selected_for_render.clone();
            let items = items_for_render.clone();
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let dialog = Dialog::new(dialog_open.clone())
                    .headline("Preferences")
                    .supporting_text("The nested Autocomplete should behave like the top overlay.")
                    .actions(vec![DialogAction::new("Done").test_id("dialog-done")])
                    .test_id("dialog")
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = Button::new("Open dialog")
                                .test_id("dialog-trigger")
                                .into_element(cx);
                            with_padding(cx, Px(24.0), trigger)
                        },
                        move |cx| {
                            vec![
                                Autocomplete::new(query.clone())
                                    .selected_value(selected.clone())
                                    .a11y_label("Dialog Autocomplete")
                                    .label("Project")
                                    .placeholder("Type to filter")
                                    .items(items.clone())
                                    .test_id("dialog-autocomplete")
                                    .into_element(cx),
                            ]
                        },
                    );
                vec![dialog]
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

    let trigger_node = semantics_node_id_by_test_id(&ui, "dialog-trigger")
        .expect("expected dialog trigger before opening");
    ui.set_focus(Some(trigger_node));
    let _ = app.models_mut().update(&dialog_open, |open| *open = true);

    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    let input_node = semantics_node_id_by_test_id(&ui, "dialog-autocomplete")
        .expect("expected autocomplete inside dialog");
    ui.set_focus(Some(input_node));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut saw_nested_popover = false;
    for _ in 0..32 {
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
        let visible_open: Vec<_> = stack
            .stack
            .iter()
            .filter(|entry| entry.open && entry.visible)
            .map(|entry| entry.kind)
            .collect();

        if visible_open.contains(&OverlayStackEntryKind::Modal)
            && visible_open.contains(&OverlayStackEntryKind::Popover)
        {
            assert_eq!(
                visible_open.last().copied(),
                Some(OverlayStackEntryKind::Popover),
                "expected the nested Autocomplete popover to be above the dialog modal layer"
            );
            saw_nested_popover = true;
            break;
        }
    }
    assert!(
        saw_nested_popover,
        "expected Autocomplete popover above Dialog"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut popover_closed = false;
    for _ in 0..60 {
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
        popover_closed = !stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.visible);
        if popover_closed {
            break;
        }
    }

    assert!(
        popover_closed,
        "expected first Escape to close the Autocomplete popover"
    );
    assert_eq!(
        app.models().get_copied(&dialog_open),
        Some(true),
        "expected first Escape to leave the Dialog open"
    );
    let input_node_after_close = semantics_node_id_by_test_id(&ui, "dialog-autocomplete")
        .expect("expected autocomplete input after closing nested popover");
    let focused_test_id = ui.semantics_snapshot().and_then(|snapshot| {
        ui.focus().and_then(|focused| {
            snapshot
                .nodes
                .iter()
                .find(|node| node.id == focused)
                .and_then(|node| node.test_id.clone())
        })
    });
    assert_eq!(
        ui.focus(),
        Some(input_node_after_close),
        "expected focus to remain on the Autocomplete input inside the Dialog; focused_test_id={focused_test_id:?}"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models().get_copied(&dialog_open),
        Some(false),
        "expected second Escape to close the Dialog"
    );
}

#[test]
fn field_overlays_inside_modal_bottom_sheet_close_before_sheet() {
    use fret_ui::element::{FlexProps, Length, SpacingLength};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{
        Autocomplete, AutocompleteItem, Button, ModalBottomSheet, Select, SelectItem, TextField,
        TextFieldVariant,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(720.0), Px(560.0)),
    );

    let sheet_open = app.models_mut().insert(false);
    let text_value = app.models_mut().insert(String::new());
    let select_value = app.models_mut().insert(Some(Arc::<str>::from("beta")));
    let autocomplete_query = app.models_mut().insert(String::new());
    let autocomplete_value = app.models_mut().insert(Some(Arc::<str>::from("beta")));

    let select_items: Arc<[SelectItem]> = vec![
        SelectItem::new("alpha", "Alpha").test_id("sheet-select-alpha"),
        SelectItem::new("beta", "Beta").test_id("sheet-select-beta"),
        SelectItem::new("gamma", "Gamma").test_id("sheet-select-gamma"),
    ]
    .into();
    let autocomplete_items: Arc<[AutocompleteItem]> = vec![
        AutocompleteItem::new("alpha", "Alpha"),
        AutocompleteItem::new("beta", "Beta"),
        AutocompleteItem::new("gamma", "Gamma"),
    ]
    .into();

    let sheet_open_for_render = sheet_open.clone();
    let text_value_for_render = text_value.clone();
    let select_value_for_render = select_value.clone();
    let autocomplete_query_for_render = autocomplete_query.clone();
    let autocomplete_value_for_render = autocomplete_value.clone();
    let select_items_for_render = select_items.clone();
    let autocomplete_items_for_render = autocomplete_items.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let sheet_open = sheet_open_for_render.clone();
            let text_value = text_value_for_render.clone();
            let select_value = select_value_for_render.clone();
            let autocomplete_query = autocomplete_query_for_render.clone();
            let autocomplete_value = autocomplete_value_for_render.clone();
            let select_items = select_items_for_render.clone();
            let autocomplete_items = autocomplete_items_for_render.clone();

            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let sheet = ModalBottomSheet::new(sheet_open.clone())
                    .test_id("sheet")
                    .into_element(
                        cx,
                        |cx| {
                            let trigger = Button::new("Open sheet")
                                .test_id("sheet-trigger")
                                .into_element(cx);
                            with_padding(cx, Px(24.0), trigger)
                        },
                        move |cx| {
                            let mut column = FlexProps::default();
                            column.direction = fret_core::Axis::Vertical;
                            column.gap = SpacingLength::Px(Px(16.0));
                            column.layout.size.width = Length::Fill;

                            let text_value = text_value.clone();
                            let select_value = select_value.clone();
                            let autocomplete_query = autocomplete_query.clone();
                            let autocomplete_value = autocomplete_value.clone();
                            let select_items = select_items.clone();
                            let autocomplete_items = autocomplete_items.clone();

                            vec![cx.flex(column, move |cx| {
                                vec![
                                    TextField::new(text_value.clone())
                                        .variant(TextFieldVariant::Outlined)
                                        .label("Project name")
                                        .placeholder("Type a name")
                                        .a11y_label("Sheet project name")
                                        .test_id("sheet-text-field")
                                        .into_element(cx),
                                    Select::new(select_value.clone())
                                        .label("Project")
                                        .placeholder("Pick one")
                                        .items(select_items.clone())
                                        .a11y_label("Sheet Select")
                                        .test_id("sheet-select")
                                        .into_element(cx),
                                    Autocomplete::new(autocomplete_query.clone())
                                        .selected_value(autocomplete_value.clone())
                                        .label("Assignee")
                                        .placeholder("Type to filter")
                                        .items(autocomplete_items.clone())
                                        .a11y_label("Sheet Autocomplete")
                                        .test_id("sheet-autocomplete")
                                        .into_element(cx),
                                ]
                            })]
                        },
                    );
                vec![sheet]
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

    let trigger_node = semantics_node_id_by_test_id(&ui, "sheet-trigger")
        .expect("expected sheet trigger before opening");
    ui.set_focus(Some(trigger_node));
    let _ = app.models_mut().update(&sheet_open, |open| *open = true);

    for _ in 0..16 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        if semantics_node_id_by_test_id(&ui, "sheet.sheet").is_some() {
            break;
        }
    }
    assert!(
        semantics_node_id_by_test_id(&ui, "sheet.sheet").is_some(),
        "expected ModalBottomSheet sheet surface"
    );

    let text_node =
        semantics_node_id_by_test_id(&ui, "sheet-text-field").expect("expected sheet TextField");
    ui.set_focus(Some(text_node));
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("Project Apollo".to_string()),
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
    assert_eq!(
        app.models().get_cloned(&text_value).as_deref(),
        Some("Project Apollo"),
        "expected TextField input inside ModalBottomSheet to update its model"
    );

    let select_node =
        semantics_node_id_by_test_id(&ui, "sheet-select").expect("expected Select inside sheet");
    ui.set_focus(Some(select_node));
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut saw_select_popover = false;
    for _ in 0..32 {
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
        let visible_open: Vec<_> = stack
            .stack
            .iter()
            .filter(|entry| entry.open && entry.visible)
            .map(|entry| entry.kind)
            .collect();
        if visible_open.contains(&OverlayStackEntryKind::Modal)
            && visible_open.contains(&OverlayStackEntryKind::Popover)
        {
            assert_eq!(
                visible_open.last().copied(),
                Some(OverlayStackEntryKind::Popover),
                "expected the Select popover to be above the bottom sheet modal layer"
            );
            saw_select_popover = true;
            break;
        }
    }
    assert!(saw_select_popover, "expected Select popover above sheet");

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut select_popover_closed = false;
    for _ in 0..60 {
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
        select_popover_closed = !stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.visible);
        if select_popover_closed {
            break;
        }
    }
    assert!(
        select_popover_closed,
        "expected first Escape to close the Select popover"
    );
    assert_eq!(
        app.models().get_copied(&sheet_open),
        Some(true),
        "expected Select Escape to leave the bottom sheet open"
    );
    let select_node_after_close =
        semantics_node_id_by_test_id(&ui, "sheet-select").expect("expected Select after close");
    assert_eq!(
        ui.focus(),
        Some(select_node_after_close),
        "expected focus to restore to Select trigger inside ModalBottomSheet"
    );

    let autocomplete_node = semantics_node_id_by_test_id(&ui, "sheet-autocomplete")
        .expect("expected Autocomplete inside sheet");
    ui.set_focus(Some(autocomplete_node));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );
    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::ArrowDown));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::ArrowDown));

    let mut saw_autocomplete_popover = false;
    for _ in 0..32 {
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
        let visible_open: Vec<_> = stack
            .stack
            .iter()
            .filter(|entry| entry.open && entry.visible)
            .map(|entry| entry.kind)
            .collect();
        if visible_open.contains(&OverlayStackEntryKind::Modal)
            && visible_open.contains(&OverlayStackEntryKind::Popover)
        {
            assert_eq!(
                visible_open.last().copied(),
                Some(OverlayStackEntryKind::Popover),
                "expected the Autocomplete popover to be above the bottom sheet modal layer"
            );
            saw_autocomplete_popover = true;
            break;
        }
    }
    assert!(
        saw_autocomplete_popover,
        "expected Autocomplete popover above sheet"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut autocomplete_popover_closed = false;
    for _ in 0..60 {
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
        autocomplete_popover_closed = !stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Popover && entry.visible);
        if autocomplete_popover_closed {
            break;
        }
    }
    assert!(
        autocomplete_popover_closed,
        "expected Escape to close the Autocomplete popover"
    );
    assert_eq!(
        app.models().get_copied(&sheet_open),
        Some(true),
        "expected Autocomplete Escape to leave the bottom sheet open"
    );
    let autocomplete_node_after_close = semantics_node_id_by_test_id(&ui, "sheet-autocomplete")
        .expect("expected Autocomplete after close");
    assert_eq!(
        ui.focus(),
        Some(autocomplete_node_after_close),
        "expected focus to remain on Autocomplete input inside ModalBottomSheet"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        false,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models().get_copied(&sheet_open),
        Some(false),
        "expected final Escape to close the ModalBottomSheet"
    );
    assert_eq!(
        ui.focus(),
        Some(trigger_node),
        "expected ModalBottomSheet to restore focus to the opener"
    );
}

#[test]
fn search_view_and_dropdown_menu_arbitrate_sibling_popovers() {
    use fret_ui::action::OnActivate;
    use fret_ui::element::{ContainerProps, FlexProps, Length, Overflow, SpacingLength};
    use fret_ui_kit::OverlayController;
    use fret_ui_material3::{
        Button, DropdownMenu, List, ListItem, MenuEntry, MenuItem, SearchView,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(760.0), Px(420.0)),
    );

    let search_open = app.models_mut().insert(false);
    let search_query = app.models_mut().insert(String::new());
    let selected_suggestion = app.models_mut().insert(Arc::<str>::from("alpha"));
    let menu_open = app.models_mut().insert(false);

    let search_open_for_render = search_open.clone();
    let search_query_for_render = search_query.clone();
    let selected_suggestion_for_render = selected_suggestion.clone();
    let menu_open_for_render = menu_open.clone();
    let render =
        move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
            let search_open = search_open_for_render.clone();
            let search_query = search_query_for_render.clone();
            let selected_suggestion = selected_suggestion_for_render.clone();
            let menu_open = menu_open_for_render.clone();

            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                let search = SearchView::new(search_open.clone(), search_query.clone())
                    .a11y_label("Search actions")
                    .placeholder("Search commands")
                    .test_id("search-menu-search")
                    .overlay_test_id("search-menu-search-panel")
                    .into_element(cx, move |cx| {
                        vec![
                            List::new(selected_suggestion.clone())
                                .a11y_label("Search suggestions")
                                .test_id("search-menu-search-suggestions")
                                .items(vec![
                                    ListItem::new("alpha", "Alpha")
                                        .test_id("search-menu-search-option-alpha"),
                                    ListItem::new("beta", "Beta")
                                        .test_id("search-menu-search-option-beta"),
                                ])
                                .into_element(cx),
                        ]
                    });

                let search = {
                    let mut container = ContainerProps::default();
                    container.layout.size.width = Length::Px(Px(420.0));
                    container.layout.overflow = Overflow::Visible;
                    cx.container(container, move |_cx| vec![search])
                };

                let toggle_menu_open: OnActivate = {
                    let menu_open = menu_open.clone();
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&menu_open, |v| *v = !*v);
                        host.request_redraw(action_cx.window);
                    })
                };

                let menu = DropdownMenu::new(menu_open.clone())
                    .a11y_label("Search actions menu")
                    .test_id("search-menu-dropdown")
                    .into_element(
                        cx,
                        move |cx| {
                            Button::new("Actions")
                                .on_activate(toggle_menu_open.clone())
                                .test_id("search-menu-menu-trigger")
                                .into_element(cx)
                        },
                        |_cx| {
                            vec![
                                MenuEntry::Item(
                                    MenuItem::new("Filter alpha").test_id("search-menu-menu-alpha"),
                                ),
                                MenuEntry::Item(
                                    MenuItem::new("Clear search").test_id("search-menu-menu-clear"),
                                ),
                            ]
                        },
                    );

                let mut row = FlexProps::default();
                row.direction = fret_core::Axis::Horizontal;
                row.align = fret_ui::element::CrossAlign::Center;
                row.gap = SpacingLength::Px(Px(12.0));
                row.layout.size.width = Length::Fill;

                let row = cx.flex(row, move |_cx| vec![search, menu]);
                vec![with_padding(cx, Px(24.0), row)]
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

    let search_node =
        semantics_node_id_by_test_id(&ui, "search-menu-search").expect("expected SearchView input");
    ui.set_focus(Some(search_node));
    run_overlay_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        true,
        |ui, app, services| render(ui, app, services),
    );

    assert_eq!(
        app.models().get_copied(&search_open),
        Some(true),
        "expected focusing SearchView to open its docked panel"
    );
    assert!(
        semantics_node_id_by_test_id(&ui, "search-menu-search-panel").is_some(),
        "expected SearchView panel while focused"
    );

    let search_node_after_open = semantics_node_id_by_test_id(&ui, "search-menu-search")
        .expect("expected current SearchView input after opening");
    assert_eq!(
        ui.focus(),
        Some(search_node_after_open),
        "expected docked SearchView to keep focus on the input after opening suggestions"
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &fret_core::Event::TextInput("alpha".to_string()),
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
    assert_eq!(
        app.models().get_cloned(&search_query).as_deref(),
        Some("alpha"),
        "expected SearchView input to keep editing before Menu opens"
    );

    let menu_trigger_node = semantics_node_id_by_test_id(&ui, "search-menu-menu-trigger")
        .expect("expected sibling DropdownMenu trigger");
    let trigger_bounds = ui
        .debug_node_visual_bounds(menu_trigger_node)
        .expect("expected sibling DropdownMenu trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(42), click_at),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_up(PointerId(42), click_at),
    );

    let mut menu_opened_and_search_closed = false;
    for _ in 0..80 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        let search_panel_exists =
            semantics_node_id_by_test_id(&ui, "search-menu-search-panel").is_some();
        let menu_exists = semantics_node_id_by_test_id(&ui, "search-menu-dropdown").is_some();
        if !search_panel_exists && menu_exists {
            menu_opened_and_search_closed = true;
            break;
        }
    }

    let search_panel_exists_after_trigger =
        semantics_node_id_by_test_id(&ui, "search-menu-search-panel").is_some();
    let menu_exists_after_trigger =
        semantics_node_id_by_test_id(&ui, "search-menu-dropdown").is_some();
    assert!(
        menu_opened_and_search_closed,
        "expected clicking sibling Menu trigger to close SearchView panel and open DropdownMenu; search_open={:?} menu_open={:?} search_panel_exists={} menu_exists={}",
        app.models().get_copied(&search_open),
        app.models().get_copied(&menu_open),
        search_panel_exists_after_trigger,
        menu_exists_after_trigger
    );
    assert_eq!(
        app.models().get_copied(&search_open),
        Some(false),
        "expected outside press on Menu trigger to close SearchView open model"
    );
    assert_eq!(
        app.models().get_copied(&menu_open),
        Some(true),
        "expected Menu trigger click to open DropdownMenu"
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack.stack.iter().any(|entry| entry.open && entry.visible),
        "expected one visible overlay after sibling Menu opens"
    );
    let menu_item = semantics_node_id_by_test_id(&ui, "search-menu-menu-alpha")
        .expect("expected initial menu item");
    assert_eq!(
        ui.focus(),
        Some(menu_item),
        "expected DropdownMenu to move focus to its first enabled item after SearchView closes"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut menu_closed = false;
    for _ in 0..60 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        menu_closed = semantics_node_id_by_test_id(&ui, "search-menu-dropdown").is_none();
        if menu_closed {
            break;
        }
    }

    assert!(menu_closed, "expected Escape to close sibling DropdownMenu");
    let menu_trigger_after_close = semantics_node_id_by_test_id(&ui, "search-menu-menu-trigger")
        .expect("expected sibling DropdownMenu trigger after close");
    assert_eq!(
        ui.focus(),
        Some(menu_trigger_after_close),
        "expected DropdownMenu Escape to restore focus to its trigger"
    );
}

#[test]
fn search_view_full_screen_blocks_sibling_menu_until_dismissed() {
    use fret_ui::action::OnActivate;
    use fret_ui::element::{ContainerProps, FlexProps, Length, Overflow, SpacingLength};
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{
        Button, DropdownMenu, List, ListItem, MenuEntry, MenuItem, SearchView,
        SearchViewPresentation,
    };

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Light, DynamicVariant::Expressive);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(760.0), Px(420.0)),
    );

    let search_open = app.models_mut().insert(false);
    let search_query = app.models_mut().insert(String::new());
    let selected_suggestion = app.models_mut().insert(Arc::<str>::from("alpha"));
    let menu_open = app.models_mut().insert(false);

    let search_open_for_render = search_open.clone();
    let search_query_for_render = search_query.clone();
    let selected_suggestion_for_render = selected_suggestion.clone();
    let menu_open_for_render = menu_open.clone();
    let render = move |ui: &mut UiTree<TestHost>,
                       app: &mut TestHost,
                       services: &mut dyn UiServices| {
        let search_open = search_open_for_render.clone();
        let search_query = search_query_for_render.clone();
        let selected_suggestion = selected_suggestion_for_render.clone();
        let menu_open = menu_open_for_render.clone();

        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            let search = SearchView::new(search_open.clone(), search_query.clone())
                .a11y_label("Full screen search actions")
                .placeholder("Search commands")
                .presentation(SearchViewPresentation::FullScreen)
                .test_id("search-full-search")
                .overlay_test_id("search-full-panel")
                .into_element(cx, move |cx| {
                    vec![
                        List::new(selected_suggestion.clone())
                            .a11y_label("Full screen search suggestions")
                            .test_id("search-full-suggestions")
                            .items(vec![
                                ListItem::new("alpha", "Alpha").test_id("search-full-option-alpha"),
                                ListItem::new("beta", "Beta").test_id("search-full-option-beta"),
                            ])
                            .into_element(cx),
                    ]
                });

            let search = {
                let mut container = ContainerProps::default();
                container.layout.size.width = Length::Px(Px(420.0));
                container.layout.overflow = Overflow::Visible;
                cx.container(container, move |_cx| vec![search])
            };

            let toggle_menu_open: OnActivate = {
                let menu_open = menu_open.clone();
                Arc::new(move |host, action_cx, _reason| {
                    let _ = host.models_mut().update(&menu_open, |v| *v = !*v);
                    host.request_redraw(action_cx.window);
                })
            };

            let menu = DropdownMenu::new(menu_open.clone())
                .a11y_label("Full screen search actions menu")
                .test_id("search-full-menu")
                .into_element(
                    cx,
                    move |cx| {
                        Button::new("Actions")
                            .on_activate(toggle_menu_open.clone())
                            .test_id("search-full-menu-trigger")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            MenuEntry::Item(
                                MenuItem::new("Filter alpha").test_id("search-full-menu-alpha"),
                            ),
                            MenuEntry::Item(
                                MenuItem::new("Clear search").test_id("search-full-menu-clear"),
                            ),
                        ]
                    },
                );

            let mut row = FlexProps::default();
            row.direction = fret_core::Axis::Horizontal;
            row.align = fret_ui::element::CrossAlign::Center;
            row.gap = SpacingLength::Px(Px(12.0));
            row.layout.size.width = Length::Fill;

            let row = cx.flex(row, move |_cx| vec![search, menu]);
            vec![with_padding(cx, Px(24.0), row)]
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

    let search_node = semantics_node_id_by_test_id(&ui, "search-full-search")
        .expect("expected full-screen SearchView underlay input");
    ui.set_focus(Some(search_node));
    for _ in 0..64 {
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

    assert_eq!(
        app.models().get_copied(&search_open),
        Some(true),
        "expected focusing full-screen SearchView to open the modal"
    );
    assert!(
        semantics_node_id_by_test_id(&ui, "search-full-panel").is_some(),
        "expected full-screen SearchView modal panel"
    );

    let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
    assert!(
        stack
            .stack
            .iter()
            .any(|entry| entry.kind == OverlayStackEntryKind::Modal && entry.open && entry.visible),
        "expected full-screen SearchView to occupy the modal overlay layer"
    );

    let header = semantics_node_id_by_test_id(&ui, "search-full-search.overlay.header")
        .expect("expected full-screen SearchView overlay header input");
    assert_eq!(
        ui.focus(),
        Some(header),
        "expected full-screen SearchView to keep editing focus in the overlay header"
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &Event::TextInput("delta".to_string()),
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
    assert_eq!(
        app.models().get_cloned(&search_query).as_deref(),
        Some("delta"),
        "expected full-screen SearchView header input to update the shared query model"
    );

    let menu_trigger_node = semantics_node_id_by_test_id(&ui, "search-full-menu-trigger")
        .expect("expected sibling DropdownMenu trigger");
    let trigger_bounds = ui
        .debug_node_visual_bounds(menu_trigger_node)
        .expect("expected sibling DropdownMenu trigger bounds");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(7), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(7), click_at));
    for _ in 0..8 {
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

    assert_eq!(
        app.models().get_copied(&menu_open),
        Some(false),
        "expected full-screen SearchView modal to block sibling menu trigger activation"
    );
    assert_eq!(
        app.models().get_copied(&search_open),
        Some(true),
        "expected blocked sibling menu click to leave full-screen SearchView open"
    );
    assert!(
        semantics_node_id_by_test_id(&ui, "search-full-panel").is_some(),
        "expected full-screen SearchView panel to remain open after blocked sibling click"
    );

    ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
    ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

    let mut search_closed = false;
    for _ in 0..80 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        search_closed = semantics_node_id_by_test_id(&ui, "search-full-panel").is_none();
        if search_closed {
            break;
        }
    }

    assert!(
        search_closed,
        "expected Escape to close full-screen SearchView"
    );
    assert_eq!(
        app.models().get_copied(&search_open),
        Some(false),
        "expected Escape to collapse full-screen SearchView open model"
    );
    assert_eq!(
        app.models().get_cloned(&search_query).as_deref(),
        Some("delta"),
        "expected closing full-screen SearchView to preserve query state"
    );

    let menu_trigger_after_close = semantics_node_id_by_test_id(&ui, "search-full-menu-trigger")
        .expect("expected sibling DropdownMenu trigger after SearchView closes");
    let trigger_bounds = ui
        .debug_node_visual_bounds(menu_trigger_after_close)
        .expect("expected sibling DropdownMenu trigger bounds after SearchView closes");
    let click_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );
    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_down(PointerId(8), click_at),
    );
    ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(8), click_at));

    let mut menu_opened = false;
    for _ in 0..80 {
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );
        menu_opened = semantics_node_id_by_test_id(&ui, "search-full-menu").is_some();
        if menu_opened {
            break;
        }
    }

    assert!(
        menu_opened,
        "expected sibling DropdownMenu to open after SearchView closes"
    );
    assert_eq!(
        app.models().get_copied(&menu_open),
        Some(true),
        "expected sibling DropdownMenu open model after SearchView dismiss"
    );
    assert!(
        semantics_node_id_by_test_id(&ui, "search-full-panel").is_none(),
        "expected opening sibling DropdownMenu not to reopen full-screen SearchView"
    );

    let first_item = semantics_node_id_by_test_id(&ui, "search-full-menu-alpha")
        .expect("expected first sibling menu item");
    assert_eq!(
        ui.focus(),
        Some(first_item),
        "expected sibling DropdownMenu to take focus after full-screen SearchView dismisses"
    );
}

#[test]
fn dialog_scrim_dismisses_without_activating_underlay() {
    use fret_ui_material3::{Button, Dialog, DialogAction};

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
            Size::new(Px(420.0), Px(320.0)),
        );

        let open = app.models_mut().insert(false);
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                let open_model = open_model.clone();
                let underlay_model = underlay_model.clone();
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let underlay_model = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            enabled: true,
                            focusable: false,
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-fullscreen")),
                                ..Default::default()
                            },
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.position = fret_ui::element::PositionStyle::Absolute;
                                l.size.width = fret_ui::element::Length::Fill;
                                l.size.height = fret_ui::element::Length::Fill;
                                l.inset = fret_ui::element::InsetStyle {
                                    top: Some(Px(0.0)).into(),
                                    right: Some(Px(0.0)).into(),
                                    bottom: Some(Px(0.0)).into(),
                                    left: Some(Px(0.0)).into(),
                                };
                                l
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_model);
                            Vec::new()
                        },
                    );

                    let dialog = Dialog::new(open_model.clone())
                        .headline("Dialog")
                        .supporting_text("Body")
                        .actions(vec![DialogAction::new("OK").test_id("dialog-ok")])
                        .test_id("dialog")
                        .into_element(
                            cx,
                            move |cx| {
                                let trigger = Button::new("Open dialog")
                                    .test_id("dialog-trigger")
                                    .into_element(cx);
                                with_padding(cx, Px(24.0), trigger)
                            },
                            |_cx| Vec::new(),
                        );
                    vec![underlay, dialog]
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
                    (node.test_id.as_deref() == Some("dialog-trigger")).then_some(node.id)
                })
            })
            .unwrap_or_else(|| panic!("expected dialog-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));
        assert_eq!(ui.focus(), Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let snapshot = ui
            .semantics_snapshot()
            .expect("expected semantics snapshot");
        assert!(
            snapshot.barrier_root.is_some(),
            "expected modal barrier root while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("dialog.scrim")),
            "expected dialog scrim node while dialog is open ({label})"
        );
        assert!(
            snapshot
                .nodes
                .iter()
                .any(|node| node.test_id.as_deref() == Some("underlay-fullscreen")),
            "expected underlay-fullscreen node while dialog is open ({label})"
        );

        let click_at = Point::new(Px(4.0), Px(4.0));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected dialog to dismiss on scrim press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dialog scrim to prevent underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dialog to restore focus to trigger after scrim dismissal ({label})"
        );
    }
}

#[test]
fn tooltip_opens_and_closes_on_hover_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    fn render_tooltip_root(
        ui: &mut UiTree<TestHost>,
        app: &mut TestHost,
        services: &mut dyn UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) -> NodeId {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = PlainTooltip::new(trigger, "Tip")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
        })
    }

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
            Size::new(Px(420.0), Px(320.0)),
        );

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
        );

        let trigger_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if stack.stack.iter().any(|entry| {
                entry.kind == OverlayStackEntryKind::Tooltip && entry.open && entry.visible
            }) {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        let unhover_at = Point::new(Px(1.0), Px(1.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), unhover_at),
        );

        let mut closed = false;
        for _ in 0..6 {
            run_overlay_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                false,
                |ui, app, services| render_tooltip_root(ui, app, services, window, bounds),
            );

            let stack = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
            if !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(closed, "expected tooltip to close after unhover ({label})");
    }
}

#[test]
fn rich_tooltip_opens_and_closes_on_hover_smoke() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, RichTooltip, TooltipProvider};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = RichTooltip::new(trigger, "Supporting text")
                        .title("Title")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
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
                if node.test_id.as_deref() == Some("tooltip-trigger") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected tooltip-trigger in semantics snapshot");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected tooltip-trigger bounds");
    let hover_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), hover_at),
    );

    let mut opened = false;
    for _ in 0..6 {
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
            .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
        {
            opened = true;
            break;
        }
    }
    assert!(opened, "expected rich tooltip to open on hover");

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move(PointerId(1), Point::new(Px(0.0), Px(0.0))),
    );

    let mut closed = false;
    for _ in 0..10 {
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
            .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
        {
            closed = true;
            break;
        }
    }
    assert!(closed, "expected rich tooltip to close after unhover");
}

#[test]
fn tooltip_does_not_open_on_touch_move() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

    let mut app = TestHost::default();
    app.set_global(PlatformCapabilities::default());
    apply_material_theme(&mut app, SchemeMode::Dark, DynamicVariant::TonalSpot);

    let window = AppWindowId::default();
    let mut services = FakeUiServices;
    let mut ui: UiTree<TestHost> = UiTree::new();
    ui.set_window(window);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(320.0)),
    );

    let render = |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
        fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
            TooltipProvider::new()
                .delay_duration_frames(0)
                .skip_delay_duration_frames(0)
                .with_elements(cx, |cx| {
                    let trigger = Button::new("Trigger")
                        .test_id("tooltip-trigger")
                        .into_element(cx);
                    let tooltip = PlainTooltip::new(trigger, "Tip")
                        .open_delay_frames(Some(0))
                        .close_delay_frames(Some(0))
                        .into_element(cx);
                    vec![with_padding(cx, Px(24.0), tooltip)]
                })
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
                if node.test_id.as_deref() == Some("tooltip-trigger") {
                    Some(node.id)
                } else {
                    None
                }
            })
        })
        .expect("expected tooltip-trigger in semantics snapshot");
    let trigger_bounds = ui
        .debug_node_visual_bounds(trigger_node)
        .expect("expected tooltip-trigger bounds");
    let hover_at = Point::new(
        Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
        Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
    );

    ui.dispatch_event(
        &mut app,
        &mut services,
        &pointer_move_touch(PointerId(1), hover_at),
    );

    for _ in 0..6 {
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
        assert!(
            !stack
                .stack
                .iter()
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible),
            "expected tooltip to remain closed when primary pointer is touch"
        );
    }
}

#[test]
fn tooltip_is_click_through_and_does_not_block_underlay_activation_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::{Button, PlainTooltip, TooltipProvider};

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

        let underlay_toggled = app.models_mut().insert(false);
        let underlay_toggled_for_render = underlay_toggled.clone();
        let render = move |ui: &mut UiTree<TestHost>,
                           app: &mut TestHost,
                           services: &mut dyn UiServices| {
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                TooltipProvider::new()
                    .delay_duration_frames(0)
                    .skip_delay_duration_frames(0)
                    .with_elements(cx, |cx| {
                        let trigger = Button::new("Trigger")
                            .test_id("tooltip-trigger")
                            .into_element(cx);
                        let tooltip = PlainTooltip::new(trigger, "Tip")
                            .open_delay_frames(Some(0))
                            .close_delay_frames(Some(0))
                            .into_element(cx);

                        let underlay_toggled = underlay_toggled_for_render.clone();
                        let underlay = cx.pressable(
                            fret_ui::element::PressableProps {
                                layout: {
                                    let mut l = fret_ui::element::LayoutStyle::default();
                                    l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                    l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                    l
                                },
                                a11y: fret_ui::element::PressableA11y {
                                    test_id: Some(std::sync::Arc::<str>::from("tooltip-underlay")),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            move |cx, _st| {
                                cx.pressable_toggle_bool(&underlay_toggled);
                                Vec::new()
                            },
                        );

                        let mut props = fret_ui::element::FlexProps::default();
                        props.direction = fret_core::Axis::Vertical;
                        props.gap = fret_ui::element::SpacingLength::Px(Px(24.0));
                        vec![cx.flex(props, move |_cx| vec![tooltip, underlay])]
                    })
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
                    if node.test_id.as_deref() == Some("tooltip-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-trigger in semantics snapshot ({label})"));
        let trigger_bounds = ui
            .debug_node_visual_bounds(trigger_node)
            .expect("expected tooltip-trigger bounds");
        let hover_at = Point::new(
            Px(trigger_bounds.origin.x.0 + trigger_bounds.size.width.0 * 0.5),
            Px(trigger_bounds.origin.y.0 + trigger_bounds.size.height.0 * 0.5),
        );

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("tooltip-underlay") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected tooltip-underlay in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected tooltip-underlay bounds");
        let click_at = Point::new(
            Px(underlay_bounds.origin.x.0 + underlay_bounds.size.width.0 * 0.5),
            Px(underlay_bounds.origin.y.0 + underlay_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_move(PointerId(1), hover_at),
        );

        let mut opened = false;
        for _ in 0..8 {
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
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                opened = true;
                break;
            }
        }
        assert!(opened, "expected tooltip to open on hover ({label})");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &pointer_down(PointerId(1), click_at),
        );
        ui.dispatch_event(&mut app, &mut services, &pointer_up(PointerId(1), click_at));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(true),
            "expected tooltip to be click-through and allow underlay activation ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(underlay_node),
            "expected underlay to receive focus when clicking through tooltip ({label})"
        );

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
                .any(|entry| entry.kind == OverlayStackEntryKind::Tooltip && entry.visible)
            {
                closed = true;
                break;
            }
        }
        assert!(
            closed,
            "expected tooltip to close after outside press without blocking underlay ({label})"
        );
    }
}

#[test]
fn dropdown_menu_dismisses_and_restores_focus_across_schemes() {
    use fret_ui_kit::{OverlayController, OverlayStackEntryKind};
    use fret_ui_material3::DropdownMenu;
    use fret_ui_material3::menu::{MenuEntry, MenuItem};

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

        let open = app.models_mut().insert(false);
        let underlay_toggled = app.models_mut().insert(false);

        let open_model = open.clone();
        let underlay_model = underlay_toggled.clone();
        let render =
            move |ui: &mut UiTree<TestHost>, app: &mut TestHost, services: &mut dyn UiServices| {
                fret_ui::declarative::render_root(ui, app, services, window, bounds, "root", |cx| {
                    let menu = DropdownMenu::new(open_model.clone())
                        .a11y_label("menu")
                        .test_id("dropdown")
                        .into_element(
                            cx,
                            |cx| {
                                cx.pressable_with_id(
                                    fret_ui::element::PressableProps {
                                        layout: {
                                            let mut l = fret_ui::element::LayoutStyle::default();
                                            l.size.width = fret_ui::element::Length::Px(Px(120.0));
                                            l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                            l
                                        },
                                        a11y: fret_ui::element::PressableA11y {
                                            test_id: Some(std::sync::Arc::<str>::from(
                                                "dropdown-trigger",
                                            )),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    |_cx, _st, _id| Vec::new(),
                                )
                            },
                            |_cx| {
                                vec![
                                    MenuEntry::Item(MenuItem::new("A").test_id("dropdown-item-a")),
                                    MenuEntry::Item(MenuItem::new("B").test_id("dropdown-item-b")),
                                    MenuEntry::Item(MenuItem::new("C").test_id("dropdown-item-c")),
                                ]
                            },
                        );

                    let underlay_toggled = underlay_model.clone();
                    let underlay = cx.pressable(
                        fret_ui::element::PressableProps {
                            layout: {
                                let mut l = fret_ui::element::LayoutStyle::default();
                                l.size.width = fret_ui::element::Length::Px(Px(160.0));
                                l.size.height = fret_ui::element::Length::Px(Px(40.0));
                                l
                            },
                            a11y: fret_ui::element::PressableA11y {
                                test_id: Some(std::sync::Arc::<str>::from("underlay-toggle")),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        move |cx, _st| {
                            cx.pressable_toggle_bool(&underlay_toggled);
                            Vec::new()
                        },
                    );

                    let mut props = fret_ui::element::FlexProps::default();
                    props.direction = fret_core::Axis::Vertical;
                    props.gap = fret_ui::element::SpacingLength::Px(Px(220.0));
                    vec![cx.flex(props, move |_cx| vec![menu, underlay])]
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
                    if node.test_id.as_deref() == Some("dropdown-trigger") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected dropdown-trigger in semantics snapshot ({label})"));
        ui.set_focus(Some(trigger_node));

        let _ = app.models_mut().update(&open, |v| *v = true);
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
        assert!(
            stack
                .stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.open),
            "expected dropdown menu overlay to be open ({label})"
        );

        ui.dispatch_event(&mut app, &mut services, &key_down(KeyCode::Escape));
        ui.dispatch_event(&mut app, &mut services, &key_up(KeyCode::Escape));

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );
        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on Escape ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on Escape ({label})"
        );

        let _ = app.models_mut().update(&open, |v| *v = true);
        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            true,
            |ui, app, services| render(ui, app, services),
        );

        let underlay_node: NodeId = ui
            .semantics_snapshot()
            .and_then(|snapshot| {
                snapshot.nodes.iter().find_map(|node| {
                    if node.test_id.as_deref() == Some("underlay-toggle") {
                        Some(node.id)
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_else(|| panic!("expected underlay-toggle in semantics snapshot ({label})"));
        let underlay_bounds = ui
            .debug_node_visual_bounds(underlay_node)
            .expect("expected underlay-toggle bounds");
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

        run_overlay_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            false,
            |ui, app, services| render(ui, app, services),
        );

        assert_eq!(
            app.models().get_copied(&open),
            Some(false),
            "expected dropdown menu to close on outside press ({label})"
        );
        assert_eq!(
            app.models().get_copied(&underlay_toggled),
            Some(false),
            "expected dropdown menu to prevent underlay activation on outside press ({label})"
        );

        let mut saw_unmount = false;
        for _ in 0..60 {
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
                saw_unmount = true;
                break;
            }
        }
        assert!(
            saw_unmount,
            "expected dropdown menu popover layer to unmount after close ({label})"
        );
        assert_eq!(
            ui.focus(),
            Some(trigger_node),
            "expected dropdown menu to restore focus to trigger on outside press ({label})"
        );
    }
}
