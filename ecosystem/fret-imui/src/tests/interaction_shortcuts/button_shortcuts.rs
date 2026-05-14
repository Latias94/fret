use super::*;

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
