use super::*;

#[test]
fn input_text_history_picker_keyboard_navigation_wraps_up_to_last_candidate() {
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
    let history = vec![
        Arc::<str>::from("first"),
        Arc::<str>::from("second"),
        Arc::<str>::from("third"),
    ];
    let picked_index = Rc::new(Cell::new(None::<usize>));
    let picked = Rc::new(RefCell::new(None::<Arc<str>>));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  picked_index_out: &Rc<Cell<Option<usize>>>,
                  picked_out: &Rc<RefCell<Option<Arc<str>>>>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.input_text_history_model_with_options(
                "imui-input-text-history-picker-keyboard.popup",
                &model,
                &history,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-history-picker-keyboard.input")),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-input-text-history-picker-keyboard")),
                    ..Default::default()
                },
            );
            picked_index_out.set(response.picked_index());
            picked_out.replace(response.picked().map(Arc::from));
        })
    };

    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);

    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.option.2",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );
    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    assert!(picker_option_active(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.option.0",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowUp,
        Modifiers::default(),
    );
    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );
    assert!(picker_option_active(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.option.2",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::NumpadEnter,
        Modifiers::default(),
    );
    app.advance_frame();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-history-picker-keyboard",
        |cx| render(cx, &picked_index_out, &picked_out),
    );

    assert_eq!(picked_index.get(), Some(2));
    assert_eq!(picked.borrow().as_deref(), Some("third"));
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("third"));
}
