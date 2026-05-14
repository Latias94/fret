use super::*;

#[test]
fn input_text_history_picker_shows_unfiltered_history_when_empty() {
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
    let history = vec![Arc::<str>::from("first"), Arc::<str>::from("second")];

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let _ = ui.input_text_history_model_with_options(
                "imui-input-text-history-picker.popup",
                &model,
                &history,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-history-picker.input")),
                        ..Default::default()
                    },
                    filter: InputTextPickerFilter::PrefixCaseInsensitive,
                    test_id: Some(Arc::from("imui-input-text-history-picker")),
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
        "imui-input-text-history-picker",
        render,
    );
    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker",
        render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker.option.0",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker.option.1",
    ));
}
