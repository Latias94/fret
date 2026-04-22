use super::*;

use fret_runtime::{
    CommandId, CommandMeta, DefaultKeybinding, PlatformFilter, WindowCommandEnabledService,
};

#[test]
fn menu_item_command_uses_command_metadata_shortcut_and_gating() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.open-file");
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
            ui.menu_item_command_with_options(
                command.clone(),
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-menu-command")),
                    ..Default::default()
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
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-menu-command"))
        .expect("menu item semantics node");
    assert_eq!(node.label.as_deref(), Some("Open File"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command.shortcut",
    ));

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}
#[test]
fn button_command_uses_command_metadata_and_gating() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.open-project");
    app.commands_mut()
        .register(command.clone(), CommandMeta::new("Open Project"));

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.button_command_with_options(
                command.clone(),
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-button-command")),
                    ..Default::default()
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
        "imui-button-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-button-command"))
        .expect("button semantics node");
    assert_eq!(node.label.as_deref(), Some("Open Project"));
    assert_eq!(node.role, fret_core::SemanticsRole::Button);

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}
#[test]
fn button_activate_shortcut_is_scoped_to_focused_button() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.shortcut.focused");
    app.commands_mut()
        .register(command.clone(), CommandMeta::new("Focused Shortcut"));

    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                ui.button_command_with_options(
                    command.clone(),
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                );
                ui.button_with_options(
                    "Other",
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut.other")),
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut",
        render,
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);
    assert!(
        app.effects.is_empty(),
        "expected unfocused shortcut to stay local to the button"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);
    app.effects.clear();

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut",
        &render,
    );
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);
    assert!(
        app.effects.is_empty(),
        "expected shortcut on another focused button to do nothing"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);
    app.effects.clear();

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut",
        &render,
    );
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}
#[test]
fn button_activate_shortcut_repeat_is_opt_in() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let default_command = CommandId::from("test.shortcut.repeat.default");
    let repeat_command = CommandId::from("test.shortcut.repeat.repeat");
    app.commands_mut()
        .register(default_command.clone(), CommandMeta::new("Default Repeat"));
    app.commands_mut()
        .register(repeat_command.clone(), CommandMeta::new("Enabled Repeat"));

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                ui.button_command_with_options(
                    default_command.clone(),
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut-repeat.default")),
                        activate_shortcut: Some(default_shortcut),
                        ..Default::default()
                    },
                );
                ui.button_command_with_options(
                    repeat_command.clone(),
                    fret_ui_kit::imui::ButtonOptions {
                        test_id: Some(Arc::from("imui-button-shortcut-repeat.repeat")),
                        activate_shortcut: Some(repeat_shortcut),
                        shortcut_repeat: true,
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-shortcut-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);
    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);
    assert_eq!(
        app.effects
            .iter()
            .filter(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: Some(target_window), command: target_command }
                        if *target_window == window && *target_command == default_command
                )
            })
            .count(),
        1,
        "expected repeat keydown to be ignored unless shortcut_repeat is enabled"
    );

    app.effects.clear();
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-shortcut-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);
    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);
    assert_eq!(
        app.effects
            .iter()
            .filter(|effect| {
                matches!(
                    effect,
                    Effect::Command { window: Some(target_window), command: target_command }
                        if *target_window == window && *target_command == repeat_command
                )
            })
            .count(),
        2,
        "expected repeat keydown to retrigger only when shortcut_repeat is enabled"
    );
}
#[test]
fn selectable_activate_shortcut_is_scoped_to_focused_item() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let shortcut = ctrl_shortcut(KeyCode::KeyK);
    let target_clicked = Rc::new(Cell::new(false));
    let other_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_clicked_out: &Rc<Cell<bool>>,
                  other_clicked_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                target_clicked_out.set(
                    ui.selectable_with_options(
                        "Target",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
                other_clicked_out.set(
                    ui.selectable_with_options(
                        "Other",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut.other")),
                            ..Default::default()
                        },
                    )
                    .clicked(),
                );
            });
        })
    };

    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected unfocused shortcut to stay local to the selectable"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(
        !target_clicked.get() && !other_clicked.get(),
        "expected shortcut on another focused selectable to do nothing"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());

    app.advance_frame();
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(!target_clicked.get());
    assert!(!other_clicked.get());

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
    let target_clicked_out = target_clicked.clone();
    let other_clicked_out = other_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut",
        |cx| render(cx, &target_clicked_out, &other_clicked_out),
    );
    assert!(target_clicked.get());
    assert!(!other_clicked.get());
}
#[test]
fn selectable_activate_shortcut_preserves_popup_arrow_nav() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
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
    let popup_id = "imui-selectable-shortcut-popup";

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let popup_open = ui.popup_open_model(popup_id);
            let is_open = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&popup_open)
                .unwrap_or(false);
            if !is_open {
                ui.open_popup_at(
                    popup_id,
                    Rect::new(Point::new(Px(48.0), Px(48.0)), Size::new(Px(1.0), Px(1.0))),
                );
            }

            assert!(ui.begin_popup_menu_with_options(
                popup_id,
                None,
                fret_ui_kit::imui::PopupMenuOptions {
                    estimated_size: Size::new(Px(160.0), Px(90.0)),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.selectable_with_options(
                        "Alpha",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.alpha")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.selectable_with_options(
                        "Beta",
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from("imui-selectable-shortcut-popup.beta")),
                            ..Default::default()
                        },
                    );
                },
            ));
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    let alpha = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-selectable-shortcut-popup.alpha",
    );
    click_at(&mut ui, &mut app, &mut services, alpha);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.alpha")
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-selectable-shortcut-popup",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = ui
        .focus()
        .and_then(|focus| {
            snap.nodes
                .iter()
                .find(|node| node.id == focus)
                .and_then(|node| node.test_id.as_deref())
        })
        .map(str::to_owned);
    assert_eq!(
        focused_test_id.as_deref(),
        Some("imui-selectable-shortcut-popup.beta")
    );
}
#[test]
fn checkbox_activate_shortcut_preserves_shift_f10_context_menu_request() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(false);
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let requested = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-checkbox-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.checkbox_model_with_options(
                        "Enabled",
                        &model,
                        fret_ui_kit::imui::CheckboxOptions {
                            test_id: Some(Arc::from("imui-checkbox-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    )
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());
}
#[test]
fn collapsing_header_activate_shortcut_is_scoped_to_focused_trigger() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let target_open = app.models_mut().insert(false);
    let other_open = app.models_mut().insert(false);
    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let target_state = Rc::new(Cell::new(false));
    let other_state = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_state_out: &Rc<Cell<bool>>,
                  other_state_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.collapsing_header_with_options(
                    "target",
                    "Target",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(target_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                    |_ui| {},
                );
                let _ = ui.collapsing_header_with_options(
                    "other",
                    "Other",
                    fret_ui_kit::imui::CollapsingHeaderOptions {
                        open: Some(other_open.clone()),
                        header_test_id: Some(Arc::from("imui-collapsing-shortcut.other")),
                        ..Default::default()
                    },
                    |_ui| {},
                );
            });

            target_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&target_open)
                    .unwrap_or_default(),
            );
            other_state_out.set(
                ui.cx_mut()
                    .app
                    .models()
                    .get_copied(&other_open)
                    .unwrap_or_default(),
            );
        })
    };

    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(!other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && !other_state.get(),
        "expected unfocused disclosure shortcut to do nothing"
    );

    let other = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.other",
    );
    click_at(&mut ui, &mut app, &mut services, other);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(
        !target_state.get() && other_state.get(),
        "expected shortcut on another disclosure trigger to leave target untouched"
    );

    let target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-collapsing-shortcut.target",
    );
    click_at(&mut ui, &mut app, &mut services, target);

    app.advance_frame();
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(target_state.get());
    assert!(other_state.get());

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
    let target_state_out = target_state.clone();
    let other_state_out = other_state.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-collapsing-shortcut",
        |cx| render(cx, &target_state_out, &other_state_out),
    );
    assert!(!target_state.get());
    assert!(other_state.get());
}
#[test]
fn tree_node_activate_shortcut_preserves_shift_f10_context_menu_request() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(260.0), Px(140.0)),
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

    let requested = Rc::new(Cell::new(false));
    let requested_out = requested.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .trigger
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .trigger
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .trigger
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-context-menu-shift-f10",
        |cx| {
            crate::imui_raw(cx, |ui| {
                requested_out.set(
                    ui.tree_node_with_options(
                        "node",
                        "Node",
                        fret_ui_kit::imui::TreeNodeOptions {
                            leaf: true,
                            test_id: Some(Arc::from("imui-tree-node-context-menu")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                        |_ui| {},
                    )
                    .trigger
                    .context_menu_requested(),
                );
            })
        },
    );
    assert!(!requested.get());
}
#[test]
fn tree_node_children_stack_vertically_inside_open_parents() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-tree-node-vertical-stacking",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.tree_node_with_options(
                    "scene",
                    "Scene",
                    fret_ui_kit::imui::TreeNodeOptions {
                        default_open: true,
                        test_id: Some(Arc::from("imui-tree-node-stack.scene")),
                        content_test_id: Some(Arc::from("imui-tree-node-stack.scene.content")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.tree_node_with_options(
                            "geometry",
                            "Geometry",
                            fret_ui_kit::imui::TreeNodeOptions {
                                default_open: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.geometry")),
                                content_test_id: Some(Arc::from(
                                    "imui-tree-node-stack.geometry.content",
                                )),
                                ..Default::default()
                            },
                            |ui| {
                                let _ = ui.tree_node_with_options(
                                    "cube",
                                    "Cube",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.cube")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                                let _ = ui.tree_node_with_options(
                                    "key-light",
                                    "Key light",
                                    fret_ui_kit::imui::TreeNodeOptions {
                                        leaf: true,
                                        level: 3,
                                        test_id: Some(Arc::from("imui-tree-node-stack.key-light")),
                                        ..Default::default()
                                    },
                                    |_ui| {},
                                );
                            },
                        );
                        let _ = ui.tree_node_with_options(
                            "postfx",
                            "Post FX",
                            fret_ui_kit::imui::TreeNodeOptions {
                                leaf: true,
                                level: 2,
                                test_id: Some(Arc::from("imui-tree-node-stack.postfx")),
                                ..Default::default()
                            },
                            |_ui| {},
                        );
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let geometry_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.geometry");
    let postfx_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.postfx");
    let cube_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.cube");
    let key_light_bounds = bounds_for_test_id(&ui, "imui-tree-node-stack.key-light");

    assert!(
        postfx_bounds.origin.y.0 >= geometry_bounds.origin.y.0 + geometry_bounds.size.height.0,
        "expected Post FX to land below Geometry, got geometry={geometry_bounds:?} postfx={postfx_bounds:?}"
    );
    assert!(
        key_light_bounds.origin.y.0 >= cube_bounds.origin.y.0 + cube_bounds.size.height.0,
        "expected Key light to land below Cube, got cube={cube_bounds:?} key_light={key_light_bounds:?}"
    );
}
