use super::*;

#[test]
fn textarea_submit_and_cancel_commands_dispatch_from_focused_multiline_field() {
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

    let model = app.models_mut().insert(String::new());
    let submit = fret_runtime::CommandId::from("editor.textarea.submit");
    let cancel = fret_runtime::CommandId::from("editor.textarea.cancel");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-command-policy",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.textarea_model_with_options(
                    &model,
                    TextAreaOptions {
                        submit_command: Some(submit.clone()),
                        cancel_command: Some(cancel.clone()),
                        test_id: Some(Arc::from("imui-textarea-command-policy")),
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
        KeyCode::Enter,
        Modifiers::default(),
    );
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::Enter);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Enter,
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
    assert_eq!(commands, vec![submit, cancel]);
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("\n"),
        "unmodified Enter should keep inserting multiline text by default"
    );
}

#[test]
fn textarea_enter_submit_policy_can_opt_into_enter_and_repeat() {
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

    let model = app.models_mut().insert(String::new());
    let submit = fret_runtime::CommandId::from("editor.textarea.submit.enter");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-enter-submit-policy",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.textarea_model_with_options(
                    &model,
                    TextAreaOptions {
                        submit_command: Some(submit.clone()),
                        submit_key: TextAreaSubmitKey::Enter,
                        submit_cancel_command_repeat: true,
                        test_id: Some(Arc::from("imui-textarea-enter-submit-policy")),
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
        KeyCode::Enter,
        Modifiers::default(),
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Enter,
        Modifiers::default(),
        true,
    );
    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::Enter);

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
    assert_eq!(commands, vec![submit.clone(), submit]);
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some(""),
        "Enter submit policy should capture Enter before textarea inserts a newline"
    );
}

#[test]
fn textarea_enter_submit_policy_consumes_repeat_when_repeat_is_disabled() {
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

    let model = app.models_mut().insert(String::new());
    let submit = fret_runtime::CommandId::from("editor.textarea.submit.enter.no-repeat");

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-enter-submit-no-repeat",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.textarea_model_with_options(
                    &model,
                    TextAreaOptions {
                        submit_command: Some(submit.clone()),
                        submit_key: TextAreaSubmitKey::Enter,
                        test_id: Some(Arc::from("imui-textarea-enter-submit-no-repeat")),
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
        KeyCode::Enter,
        Modifiers::default(),
    );
    key_down_with_repeat(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Enter,
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
    assert_eq!(commands, vec![submit]);
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some(""),
        "repeated Enter should be consumed when Enter-submit repeat is disabled"
    );
}
