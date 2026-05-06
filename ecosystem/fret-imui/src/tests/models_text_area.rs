use super::*;
use fret_ui_kit::imui::{ButtonOptions, TextAreaOptions, TextAreaSubmitKey};

#[test]
fn textarea_read_only_blocks_text_input_and_keeps_changed_false() {
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

    let model = app.models_mut().insert(String::from("locked\narea"));
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-textarea-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );
    assert!(!changed.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "!");

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-read-only",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            read_only: true,
                            test_id: Some(Arc::from("imui-textarea-read-only")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(!changed.get());
    assert_eq!(
        app.models().get_cloned(&model).as_deref(),
        Some("locked\narea")
    );
}

#[test]
fn textarea_tab_key_does_not_insert_by_default() {
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
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-default",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            test_id: Some(Arc::from("imui-textarea-tab-default")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-default",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            test_id: Some(Arc::from("imui-textarea-tab-default")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(!changed.get());
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some(""));
}

#[test]
fn textarea_allow_tab_input_inserts_tab_and_reports_changed() {
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
    let changed = Rc::new(Cell::new(false));

    let changed_out = changed.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-allowed",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            allow_tab_input: true,
                            test_id: Some(Arc::from("imui-textarea-tab-allowed")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Tab,
        Modifiers::default(),
    );

    app.advance_frame();
    let changed_out = changed.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-tab-allowed",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(
                    ui.textarea_model_with_options(
                        &model,
                        TextAreaOptions {
                            allow_tab_input: true,
                            test_id: Some(Arc::from("imui-textarea-tab-allowed")),
                            ..Default::default()
                        },
                    )
                    .changed(),
                );
            })
        },
    );

    assert!(changed.get());
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("\t"));
}

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

#[test]
fn textarea_model_reports_changed_once_after_text_input() {
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

    let model = app.models_mut().insert(String::new());

    let changed = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let changed_out = changed.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert!(text.borrow().is_empty());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    text_input_event(&mut ui, &mut app, &mut services, "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(changed.get());
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");

    app.advance_frame();
    let changed_out = changed.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea",
        |cx| {
            crate::imui_raw(cx, |ui| {
                changed_out.set(ui.textarea_model(&model).changed());
                let current = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .unwrap_or_default();
                text_out.replace(current);
            })
        },
    );
    assert!(!changed.get());
    assert_eq!(text.borrow().as_str(), "line-1\nline-2");
}

#[test]
fn textarea_lifecycle_tracks_focus_edit_and_blur_edges() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app.models_mut().insert(String::new());
    let activated = Rc::new(Cell::new(false));
    let deactivated = Rc::new(Cell::new(false));
    let edited = Rc::new(Cell::new(false));
    let after_edit = Rc::new(Cell::new(false));
    let text = Rc::new(RefCell::new(String::new()));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  activated_out: &Rc<Cell<bool>>,
                  deactivated_out: &Rc<Cell<bool>>,
                  edited_out: &Rc<Cell<bool>>,
                  after_edit_out: &Rc<Cell<bool>>,
                  text_out: &Rc<RefCell<String>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let resp = ui.textarea_model_with_options(
                    &model,
                    TextAreaOptions {
                        test_id: Some(Arc::from("imui-textarea-lifecycle")),
                        ..Default::default()
                    },
                );
                activated_out.set(resp.activated());
                deactivated_out.set(resp.deactivated());
                edited_out.set(resp.edited());
                after_edit_out.set(resp.deactivated_after_edit());

                let _ = ui.button_with_options(
                    "Blur target",
                    ButtonOptions {
                        test_id: Some(Arc::from("imui-textarea-lifecycle.blur-target")),
                        ..Default::default()
                    },
                );
            });

            let current = ui
                .cx_mut()
                .app
                .models()
                .get_cloned(&model)
                .unwrap_or_default();
            text_out.replace(current);
        })
    };

    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    let input = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(activated.get());
    assert!(!deactivated.get());
    assert!(!edited.get());
    assert!(!after_edit.get());
    assert!(text.borrow().is_empty());

    text_input_event(&mut ui, &mut app, &mut services, "hello\nworld");

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(!deactivated.get());
    assert!(edited.get());
    assert!(!after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello\nworld");

    let blur_target = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-textarea-lifecycle.blur-target",
    );
    click_at(&mut ui, &mut app, &mut services, blur_target);

    app.advance_frame();
    let activated_out = activated.clone();
    let deactivated_out = deactivated.clone();
    let edited_out = edited.clone();
    let after_edit_out = after_edit.clone();
    let text_out = text.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-textarea-lifecycle",
        |cx| {
            render(
                cx,
                &activated_out,
                &deactivated_out,
                &edited_out,
                &after_edit_out,
                &text_out,
            )
        },
    );
    assert!(!activated.get());
    assert!(deactivated.get());
    assert!(!edited.get());
    assert!(after_edit.get());
    assert_eq!(text.borrow().as_str(), "hello\nworld");
}
