use super::*;
use fret_ui_kit::imui::InputTextOptions;

#[test]
fn input_text_completion_command_dispatches_on_unmodified_tab() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let completion = fret_runtime::CommandId::from("editor.complete");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-command",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        completion_command: Some(completion.clone()),
                        test_id: Some(Arc::from("imui-input-text-completion-command")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    app.effects.clear();
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    assert!(
        app.effects.iter().any(|effect| {
            matches!(
                effect,
                Effect::Command {
                    window: Some(target_window),
                    command
                } if *target_window == window && command == &completion
            )
        }),
        "expected focused InputText Tab to dispatch the completion command"
    );
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some(""));
}

#[test]
fn input_text_history_commands_dispatch_on_unmodified_arrows_without_default_repeat() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let previous = fret_runtime::CommandId::from("editor.history.previous");
    let next = fret_runtime::CommandId::from("editor.history.next");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-commands",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        history_previous_command: Some(previous.clone()),
                        history_next_command: Some(next.clone()),
                        test_id: Some(Arc::from("imui-input-text-history-commands")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    app.effects.clear();
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowUp,
        Modifiers::default(),
    );
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowUp,
        Modifiers::default(),
        true,
    );

    let commands: Vec<_> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Command {
                window: Some(target_window),
                command,
            } if *target_window == window => Some(command.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(commands, vec![previous, next]);
}

#[test]
fn input_text_undo_redo_commands_dispatch_on_focused_shortcuts_without_default_repeat() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let undo = fret_runtime::CommandId::from("editor.undo");
    let redo = fret_runtime::CommandId::from("editor.redo");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-undo-redo-commands",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        undo_command: Some(undo.clone()),
                        redo_command: Some(redo.clone()),
                        test_id: Some(Arc::from("imui-input-text-undo-redo-commands")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    app.effects.clear();
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyZ);
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyY);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyZ,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyZ,
        ctrl_modifiers(),
        true,
    );

    let commands: Vec<_> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Command {
                window: Some(target_window),
                command,
            } if *target_window == window => Some(command.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(commands, vec![undo, redo.clone(), redo]);
}

#[test]
fn input_text_policy_commands_can_opt_into_repeat() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(140.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let completion = fret_runtime::CommandId::from("editor.complete.repeat");
    let previous = fret_runtime::CommandId::from("editor.history.previous.repeat");
    let undo = fret_runtime::CommandId::from("editor.undo.repeat");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-command-repeat",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.input_text_model_with_options(
                    &model,
                    InputTextOptions {
                        completion_command: Some(completion.clone()),
                        history_previous_command: Some(previous.clone()),
                        undo_command: Some(undo.clone()),
                        completion_command_repeat: true,
                        history_command_repeat: true,
                        undo_redo_command_repeat: true,
                        test_id: Some(Arc::from("imui-input-text-command-repeat")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    app.effects.clear();
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
        true,
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::KeyZ,
        ctrl_modifiers(),
        true,
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowUp,
        Modifiers::default(),
        true,
    );

    let commands: Vec<_> = app
        .effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Command {
                window: Some(target_window),
                command,
            } if *target_window == window => Some(command.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(commands, vec![completion, undo, previous]);
}
