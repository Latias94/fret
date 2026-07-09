use super::*;

fn assert_history_active_descendant(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    expected_option_test_id: &str,
) {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input_node = snap
        .nodes
        .iter()
        .find(|node| {
            node.test_id.as_deref() == Some("imui-input-text-history-picker-keyboard.input")
        })
        .expect("expected history picker input semantics node");
    let active_option = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(expected_option_test_id))
        .expect("expected active history picker option semantics node");
    let popup_panel = snap
        .nodes
        .iter()
        .find(|node| {
            node.test_id.as_deref()
                == Some("imui-popup-imui-input-text-history-picker-keyboard.popup")
        })
        .expect("expected history picker popup panel semantics node");

    assert_eq!(input_node.role, SemanticsRole::ComboBox);
    assert!(input_node.flags.expanded);
    assert_eq!(input_node.active_descendant, Some(active_option.id));
    assert!(input_node.controls.contains(&popup_panel.id));
}

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
    assert_history_active_descendant(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.option.0",
    );
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
    assert_history_active_descendant(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-history-picker-keyboard.option.2",
    );
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
