use super::*;

use fret_runtime::{CommandId, CommandMeta, DefaultKeybinding, PlatformFilter};
use fret_ui_kit::primitives::menu::pointer_grace_intent;

fn current_focus_test_id(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
) -> Option<String> {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let focus = ui.focus()?;
    let snap = ui.semantics_snapshot()?;
    snap.nodes
        .iter()
        .find(|node| node.id == focus)
        .and_then(|node| node.test_id.as_deref().map(str::to_owned))
}

fn find_grace_corridor_transition_points(
    reference: Rect,
    sibling: Rect,
    floating: Rect,
) -> Option<(Point, Point)> {
    let geometry = pointer_grace_intent::PointerGraceIntentGeometry {
        reference,
        floating,
    };
    let reference_right = reference.origin.x.0 + reference.size.width.0;
    let sibling_right = sibling.origin.x.0 + sibling.size.width.0;
    let sibling_bottom = sibling.origin.y.0 + sibling.size.height.0;

    for exit_y in (reference.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
        for exit_x in
            (reference.origin.x.0.floor() as i32)..=((reference_right + 24.0).ceil() as i32)
        {
            let exit = Point::new(Px(exit_x as f32), Px(exit_y as f32));
            if reference.contains(exit) || sibling.contains(exit) || floating.contains(exit) {
                continue;
            }

            let Some(intent) =
                pointer_grace_intent::grace_intent_from_exit_point(exit, geometry, Px(5.0))
            else {
                continue;
            };

            for y in (sibling.origin.y.0.floor() as i32)..=(sibling_bottom.ceil() as i32) {
                for x in (sibling.origin.x.0.floor() as i32)..=(sibling_right.ceil() as i32) {
                    let candidate = Point::new(Px(x as f32), Px(y as f32));
                    if !sibling.contains(candidate) {
                        continue;
                    }

                    let moving_towards = match intent.side {
                        pointer_grace_intent::GraceSide::Right => candidate.x.0 > exit.x.0,
                        pointer_grace_intent::GraceSide::Left => candidate.x.0 < exit.x.0,
                    };
                    if moving_towards
                        && pointer_grace_intent::is_pointer_in_grace_area(candidate, intent)
                    {
                        return Some((exit, candidate));
                    }
                }
            }
        }
    }

    None
}

fn pending_nonrepeating_timer_tokens_after(
    app: &TestHost,
    after: std::time::Duration,
) -> Vec<TimerToken> {
    app.effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::SetTimer {
                token,
                after: effect_after,
                repeat,
                ..
            } if repeat.is_none() && *effect_after == after => Some(*token),
            _ => None,
        })
        .collect()
}

#[test]
fn begin_menu_helper_toggles_popup_and_closes_after_command_activate() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.begin-menu.open");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Open File").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyO,
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let open = ui.popup_open_model("file");
                            ui.menu_item_command_with_options(
                                command.clone(),
                                MenuItemOptions {
                                    close_popup: Some(open),
                                    test_id: Some(Arc::from("imui-begin-menu.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    );
    click_at(&mut ui, &mut app, &mut services, item);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));
}
#[test]
fn begin_menu_activate_shortcut_is_scoped_to_focused_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-shortcut.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-shortcut.file")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-shortcut.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "edit",
                        "Edit",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-shortcut.edit")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-shortcut.edit.copy")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-shortcut",
        render,
    );

    let _edit_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.edit",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-shortcut",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.edit.copy",
    ));

    let _file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.file",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-shortcut",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-shortcut.edit.copy",
    ));
}

#[test]
fn begin_menu_activate_shortcut_keyboard_open_focuses_first_item_and_escape_restores_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-focus.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-focus.file")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-focus.file.open")),
                                    ..Default::default()
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Save",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-focus.file.save")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-focus",
        render,
    );

    let file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-focus.file",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-focus",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-focus.file.open",
    ));

    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-focus.file.open"))
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-focus",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-focus.file.open",
    ));
    assert_eq!(ui.focus(), Some(file_node));
}

#[test]
fn begin_menu_arrow_down_opens_menu_and_focuses_first_item() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-arrow-open.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-open.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-open.file.open",
                                    )),
                                    ..Default::default()
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Save",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-open.file.save",
                                    )),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-open",
        render,
    );

    let _file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-open.file",
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-open",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-open.file.open",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-open.file.open"))
    );
}

#[test]
fn begin_menu_horizontal_arrows_switch_active_top_level_menu() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-arrow-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-switch.file.open",
                                    )),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "edit",
                        "Edit",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-switch.edit")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-switch.edit.copy",
                                    )),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        render,
    );

    let _file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file",
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowRight,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.edit.copy"))
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowLeft,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );
}

#[test]
fn begin_menu_activate_shortcut_repeat_is_opt_in() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-repeat.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file-default",
                        "Default",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-repeat.default")),
                            activate_shortcut: Some(default_shortcut),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-repeat.default.item")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "file-repeat",
                        "Repeat",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-repeat.repeat")),
                            activate_shortcut: Some(repeat_shortcut),
                            shortcut_repeat: true,
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-repeat.repeat.item")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default.item",
    ));

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-menu-repeat.default.item",
        ),
        "expected repeated keydown to leave default shortcut trigger open"
    );

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat.item",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-menu-repeat.repeat.item",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
#[test]
fn begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.begin-submenu.open-recent");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Recent Project").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyR,
                Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let file_open = Rc::new(Cell::new(false));
    let recent_open = Rc::new(Cell::new(false));
    let file_open_out = file_open.clone();
    let recent_open_out = recent_open.clone();
    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-submenu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-submenu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent_open = ui.popup_open_model("recent");
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-begin-submenu.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_command_with_options(
                                        command.clone(),
                                        MenuItemOptions {
                                            close_popup: Some(recent_open),
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );

            let file_popup = ui.popup_open_model("file");
            let recent_popup = ui.popup_open_model("recent");
            file_open_out.set(
                ui.cx_mut()
                    .read_model(&file_popup, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false),
            );
            recent_open_out.set(
                ui.cx_mut()
                    .read_model(
                        &recent_popup,
                        fret_ui::Invalidation::Paint,
                        |_app, value| *value,
                    )
                    .unwrap_or(false),
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!file_open.get());
    assert!(!recent_open.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(!recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert_eq!(recent_node.role, fret_core::SemanticsRole::MenuItem);
    assert!(!recent_node.flags.expanded);

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    );
    click_at(&mut ui, &mut app, &mut services, recent_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(
        file_open.get(),
        "expected parent menu to remain open (file_open={} recent_open={})",
        file_open.get(),
        recent_open.get()
    );
    assert!(
        recent_open.get(),
        "expected submenu to remain open (file_open={} recent_open={})",
        file_open.get(),
        recent_open.get()
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert!(recent_node.flags.expanded);

    let project_item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    );
    click_at(&mut ui, &mut app, &mut services, project_item);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));
}
#[test]
fn begin_menu_helper_hover_switches_top_level_popup_after_trigger_hover_delay() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-menu-hover-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-hover-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-menu-hover-switch.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "edit",
                        "Edit",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-hover-switch.edit")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-menu-hover-switch.edit.copy")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit.copy",
    ));

    app.effects.clear();
    let edit_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        edit_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected hover-switch timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file.open",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit.copy",
    ));
}
#[test]
fn begin_submenu_helper_hover_opens_submenu_after_pointer_entry() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-hover-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-hover-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-hover-switch.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-hover-switch.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-hover-switch.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-hover-switch.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-hover-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.history",
    ));

    app.effects.clear();
    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-hover-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.history.yesterday",
    ));
}
#[test]
fn begin_submenu_helper_hover_switches_sibling_after_open_delay() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let recent_open = Rc::new(Cell::new(false));
    let history_open = Rc::new(Cell::new(false));
    let history_hovered = Rc::new(Cell::new(false));
    let history_hovered_raw = Rc::new(Cell::new(false));
    let history_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let recent_open = recent_open.clone();
        let history_open = history_open.clone();
        let history_hovered = history_hovered.clone();
        let history_hovered_raw = history_hovered_raw.clone();
        let history_hovered_raw_below_barrier = history_hovered_raw_below_barrier.clone();
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-sibling-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-sibling-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-sibling-switch.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-sibling-switch.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            recent_open.set(recent.open());
                            let history = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-sibling-switch.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-sibling-switch.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            history_open.set(history.open());
                            history_hovered.set(history.trigger.core.hovered);
                            history_hovered_raw.set(history.trigger.pointer_hovered_raw);
                            history_hovered_raw_below_barrier
                                .set(history.trigger.pointer_hovered_raw_below_barrier);
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.history.yesterday",
    ));

    let history_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.history",
    );
    let clear_grace_point = Point::new(
        Px((history_trigger.x.0 - 120.0).max(4.0)),
        history_trigger.y,
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        clear_grace_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        history_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected sibling hover to dispatch a timer");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-submenu-sibling-switch.file.recent.project",
        ),
        "expected the previous submenu to close after sibling hover-switch (recent_open={} history_open={} history_hovered={} history_hovered_raw={} history_hovered_raw_below_barrier={})",
        recent_open.get(),
        history_open.get(),
        history_hovered.get(),
        history_hovered_raw.get(),
        history_hovered_raw_below_barrier.get()
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-submenu-sibling-switch.file.history.yesterday",
        ),
        "expected the sibling submenu to open after hover-switch (recent_open={} history_open={} history_hovered={} history_hovered_raw={} history_hovered_raw_below_barrier={})",
        recent_open.get(),
        history_open.get(),
        history_hovered.get(),
        history_hovered_raw.get(),
        history_hovered_raw_below_barrier.get()
    );
}

#[test]
fn begin_submenu_helper_defers_sibling_switch_inside_grace_corridor() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let history_hovered = Rc::new(Cell::new(false));
    let history_hovered_raw = Rc::new(Cell::new(false));
    let history_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let history_hovered = history_hovered.clone();
        let history_hovered_raw = history_hovered_raw.clone();
        let history_hovered_raw_below_barrier = history_hovered_raw_below_barrier.clone();
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-grace-corridor.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-grace-corridor.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let history = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            history_hovered.set(history.trigger.core.hovered);
                            history_hovered_raw.set(history.trigger.pointer_hovered_raw);
                            history_hovered_raw_below_barrier
                                .set(history.trigger.pointer_hovered_raw_below_barrier);
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let recent_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.recent");
    let history_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.history");
    let recent_popup_bounds = bounds_for_test_id(&ui, "imui-popup-recent");
    let (grace_exit_point, history_grace_point) =
        find_grace_corridor_transition_points(recent_bounds, history_bounds, recent_popup_bounds)
            .expect("expected a history point inside the submenu grace corridor");

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        grace_exit_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        history_grace_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    assert!(
        history_hovered.get()
            || history_hovered_raw.get()
            || history_hovered_raw_below_barrier.get(),
        "expected pointer to hit the sibling trigger inside grace corridor (exit={grace_exit_point:?} hit={history_grace_point:?})"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let grace_timeout =
        fret_ui_kit::primitives::menu::sub::MenuSubmenuConfig::default().pointer_grace_timeout;
    let pending = pending_nonrepeating_timer_tokens_after(&app, grace_timeout);
    let dispatched = dispatch_timer_tokens(&mut ui, &mut app, &mut services, &pending);
    assert!(
        dispatched > 0,
        "expected pointer grace timeout timer to be present"
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));
}

#[test]
fn begin_submenu_activate_shortcut_is_scoped_to_focused_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-submenu-shortcut.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-submenu-shortcut.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-submenu-shortcut.file.recent",
                                    )),
                                    activate_shortcut: Some(shortcut),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Alpha",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu-shortcut.file.recent.alpha",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-submenu-shortcut.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu-shortcut.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-shortcut",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-shortcut",
        render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.recent",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.history",
    ));

    let _history_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.history",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-shortcut",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.recent.alpha",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.history.yesterday",
    ));

    let _recent_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.recent",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-shortcut",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.recent.alpha",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-shortcut.file.history.yesterday",
    ));
}
#[test]
fn begin_submenu_activate_shortcut_repeat_is_opt_in() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-submenu-repeat.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-submenu-repeat.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent-default",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-submenu-repeat.file.default",
                                    )),
                                    activate_shortcut: Some(default_shortcut),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Alpha",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu-repeat.file.default.item",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.begin_submenu_with_options(
                                "recent-repeat",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-submenu-repeat.file.repeat",
                                    )),
                                    activate_shortcut: Some(repeat_shortcut),
                                    shortcut_repeat: true,
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu-repeat.file.repeat.item",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.default",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.repeat",
    ));

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.default.item",
    ));
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-submenu-repeat.file.default.item",
        ),
        "expected repeated keydown to leave default submenu trigger open"
    );

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.repeat.item",
    ));
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu-repeat.file.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-submenu-repeat.file.repeat.item",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
#[test]
fn menu_and_submenu_helpers_report_toggle_and_trigger_edges() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let menu_open = Rc::new(Cell::new(false));
    let menu_opened = Rc::new(Cell::new(false));
    let menu_closed = Rc::new(Cell::new(false));
    let menu_activated = Rc::new(Cell::new(false));
    let menu_deactivated = Rc::new(Cell::new(false));
    let submenu_open = Rc::new(Cell::new(false));
    let submenu_opened = Rc::new(Cell::new(false));
    let submenu_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let menu_open = menu_open.clone();
        let menu_opened = menu_opened.clone();
        let menu_closed = menu_closed.clone();
        let menu_activated = menu_activated.clone();
        let menu_deactivated = menu_deactivated.clone();
        let submenu_open = submenu_open.clone();
        let submenu_opened = submenu_opened.clone();
        let submenu_clicked = submenu_clicked.clone();

        crate::imui_raw(cx, move |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-menu-response.root")),
                    ..Default::default()
                },
                |ui| {
                    let menu = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-response.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let submenu = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-menu-response.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-menu-response.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            submenu_open.set(submenu.open());
                            submenu_opened.set(submenu.opened());
                            submenu_clicked.set(submenu.clicked());
                        },
                    );
                    menu_open.set(menu.open());
                    menu_opened.set(menu.opened());
                    menu_closed.set(menu.closed());
                    menu_activated.set(menu.trigger.activated());
                    menu_deactivated.set(menu.trigger.deactivated());
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        render,
    );
    assert!(!menu_open.get());
    assert!(!menu_opened.get());
    assert!(!menu_closed.get());
    assert!(!menu_activated.get());
    assert!(!menu_deactivated.get());
    assert!(!submenu_open.get());
    assert!(!submenu_opened.get());
    assert!(!submenu_clicked.get());

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(!menu_open.get());
    assert!(menu_activated.get());
    assert!(!menu_deactivated.get());

    pointer_up_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(menu_open.get());
    assert!(menu_opened.get());
    assert!(menu_deactivated.get());
    assert!(!submenu_open.get());

    let submenu_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file.recent",
    );
    click_at(&mut ui, &mut app, &mut services, submenu_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(menu_open.get());
    assert!(submenu_open.get());
    assert!(submenu_opened.get());
    assert!(submenu_clicked.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file.recent.project",
    ));
}
#[test]
fn tab_bar_helper_switches_selected_panel_and_updates_selection_model() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let selected_model = app.models_mut().insert(Some(Arc::<str>::from("inspector")));
    let selected_out = Rc::new(RefCell::new(None::<Arc<str>>));
    let selected_out_render = selected_out.clone();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-bar-interaction.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-bar-interaction.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-bar-interaction.scene.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-bar-interaction.inspector")),
                            panel_test_id: Some(Arc::from(
                                "imui-tab-bar-interaction.inspector.panel",
                            )),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            let selected = ui
                .cx_mut()
                .read_model(
                    &selected_model,
                    fret_ui::Invalidation::Paint,
                    |_app, value| value.clone(),
                )
                .unwrap_or(None);
            selected_out_render.replace(selected);
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));

    let scene_tab = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene",
    );
    click_at(&mut ui, &mut app, &mut services, scene_tab);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.scene.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-bar-interaction.inspector.panel",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let scene_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.scene"))
        .expect("scene tab semantics node");
    let inspector_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-tab-bar-interaction.inspector"))
        .expect("inspector tab semantics node");
    assert!(scene_node.flags.selected);
    assert!(!inspector_node.flags.selected);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-bar-interaction",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
}
#[test]
fn tab_bar_helper_reports_selected_change_and_trigger_edges() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let selected_model = app.models_mut().insert(Some(Arc::<str>::from("inspector")));
    let selected_id = Rc::new(RefCell::new(None::<String>));
    let selected_changed = Rc::new(Cell::new(false));
    let scene_clicked = Rc::new(Cell::new(false));
    let scene_activated = Rc::new(Cell::new(false));
    let scene_deactivated = Rc::new(Cell::new(false));
    let scene_selected = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let selected_id = selected_id.clone();
        let selected_changed = selected_changed.clone();
        let scene_clicked = scene_clicked.clone();
        let scene_activated = scene_activated.clone();
        let scene_deactivated = scene_deactivated.clone();
        let scene_selected = scene_selected.clone();
        let selected_model = selected_model.clone();

        crate::imui_raw(cx, move |ui| {
            let tabs = ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-response.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-response.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-response.scene.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-response.inspector")),
                            panel_test_id: Some(Arc::from("imui-tab-response.inspector.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            selected_id.replace(tabs.selected_id().map(str::to_owned));
            selected_changed.set(tabs.selected_changed());
            if let Some(scene) = tabs.trigger("scene") {
                scene_clicked.set(scene.clicked());
                scene_activated.set(scene.activated());
                scene_deactivated.set(scene.deactivated());
                scene_selected.set(scene.selected());
            }
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("inspector"));
    assert!(!selected_changed.get());
    assert!(!scene_clicked.get());
    assert!(!scene_activated.get());
    assert!(!scene_deactivated.get());
    assert!(!scene_selected.get());

    let scene_tab = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-response.scene",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, scene_tab);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("inspector"));
    assert!(!selected_changed.get());
    assert!(scene_activated.get());
    assert!(!scene_clicked.get());
    assert!(!scene_deactivated.get());
    assert!(!scene_selected.get());

    pointer_up_at(&mut ui, &mut app, &mut services, scene_tab);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("scene"));
    assert!(selected_changed.get());
    assert!(scene_clicked.get());
    assert!(scene_deactivated.get());
    assert!(scene_selected.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-response.scene.panel",
    ));

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-response",
        &render,
    );
    assert_eq!(selected_id.borrow().as_deref(), Some("scene"));
    assert!(!selected_changed.get());
    assert!(!scene_clicked.get());
    assert!(!scene_activated.get());
    assert!(!scene_deactivated.get());
    assert!(scene_selected.get());
}
#[test]
fn tab_item_activate_shortcut_is_scoped_to_focused_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let selected_model = app.models_mut().insert(Some(Arc::<str>::from("inspector")));
    let selected_out = Rc::new(RefCell::new(None::<Arc<str>>));
    let selected_out_render = selected_out.clone();
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.tab_bar_with_options(
                "workspace",
                fret_ui_kit::imui::TabBarOptions {
                    selected: Some(selected_model.clone()),
                    test_id: Some(Arc::from("imui-tab-shortcut.root")),
                    ..Default::default()
                },
                |tabs| {
                    tabs.begin_tab_item_with_options(
                        "scene",
                        "Scene",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-shortcut.scene")),
                            panel_test_id: Some(Arc::from("imui-tab-shortcut.scene.panel")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Scene Panel");
                        },
                    );
                    tabs.begin_tab_item_with_options(
                        "inspector",
                        "Inspector",
                        fret_ui_kit::imui::TabItemOptions {
                            test_id: Some(Arc::from("imui-tab-shortcut.inspector")),
                            panel_test_id: Some(Arc::from("imui-tab-shortcut.inspector.panel")),
                            ..Default::default()
                        },
                        |ui| {
                            ui.text("Inspector Panel");
                        },
                    );
                },
            );

            let selected = ui
                .cx_mut()
                .read_model(
                    &selected_model,
                    fret_ui::Invalidation::Paint,
                    |_app, value| value.clone(),
                )
                .unwrap_or(None);
            selected_out_render.replace(selected);
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));

    let inspector_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector",
    );
    ui.set_focus(Some(inspector_node));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("inspector"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene.panel",
    ));

    let scene_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene",
    );
    ui.set_focus(Some(scene_node));
    assert_eq!(ui.focus(), Some(scene_node));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tab-shortcut",
        |cx| render(cx),
    );
    assert_eq!(selected_out.borrow().as_deref(), Some("scene"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.scene.panel",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-tab-shortcut.inspector.panel",
    ));
}
