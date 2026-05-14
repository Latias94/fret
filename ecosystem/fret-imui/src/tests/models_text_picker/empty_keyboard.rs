use super::*;

#[test]
fn input_text_picker_keyboard_navigation_does_not_consume_enter_without_candidates() {
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

    let model = app.models_mut().insert(String::new());
    let candidates: Vec<Arc<str>> = Vec::new();
    let submit = fret_runtime::CommandId::from("imui.test.submit-empty-picker");

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let _ = ui.input_text_completion_model_with_options(
                "imui-input-text-picker-empty-keyboard.popup",
                &model,
                &candidates,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-picker-empty-keyboard.input")),
                        submit_command: Some(submit.clone()),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-input-text-picker-empty-keyboard")),
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
        "imui-input-text-picker-empty-keyboard",
        render,
    );
    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-picker-empty-keyboard.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Enter,
        Modifiers::default(),
    );

    assert!(app.effects.iter().any(|effect| matches!(
        effect,
        Effect::Command {
            window: Some(target_window),
            command,
        } if *target_window == window && command == &submit
    )));
}
