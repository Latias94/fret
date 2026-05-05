use super::*;
use fret_ui_kit::imui::InputTextOptions;
use fret_ui_kit::imui::InputTextPickerFilter;
use fret_ui_kit::imui::InputTextPickerOptions;

#[test]
fn input_text_completion_picker_filters_popup_and_commits_clicked_candidate() {
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
    let candidates = vec![
        Arc::<str>::from("Alpha"),
        Arc::<str>::from("Beta"),
        Arc::<str>::from("Gamma"),
    ];
    let changed = Rc::new(Cell::new(false));
    let picked_index = Rc::new(Cell::new(None::<usize>));
    let picked = Rc::new(RefCell::new(None::<Arc<str>>));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  changed_out: &Rc<Cell<bool>>,
                  picked_index_out: &Rc<Cell<Option<usize>>>,
                  picked_out: &Rc<RefCell<Option<Arc<str>>>>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.input_text_completion_model_with_options(
                "imui-input-text-completion-picker.popup",
                &model,
                &candidates,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-completion-picker.input")),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-input-text-completion-picker")),
                    ..Default::default()
                },
            );
            changed_out.set(response.changed());
            picked_index_out.set(response.picked_index());
            picked_out.replace(response.picked.clone());
        })
    };

    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(!changed.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.option.0",
    ));

    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);
    text_input_event(&mut ui, &mut app, &mut services, "be");

    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.option.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.option.1",
    ));

    let option = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.option.0",
    );
    click_at(&mut ui, &mut app, &mut services, option);

    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );

    assert!(changed.get());
    assert_eq!(picked_index.get(), Some(1));
    assert_eq!(picked.borrow().as_deref(), Some("Beta"));
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("Beta"));

    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker.option.0",
    ));
}

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

fn picker_option_selected(
    ui: &mut UiTree<TestHost>,
    app: &mut TestHost,
    services: &mut FakeTextService,
    bounds: Rect,
    test_id: &str,
) -> bool {
    ui.request_semantics_snapshot();
    ui.layout_all(app, services, bounds, 1.0);
    ui.semantics_snapshot()
        .expect("semantics snapshot")
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some(test_id))
        .unwrap_or_else(|| panic!("expected semantics node with test_id {test_id:?}"))
        .flags
        .selected
}

#[test]
fn input_text_completion_picker_keyboard_navigation_commits_active_candidate() {
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
    let candidates = vec![
        Arc::<str>::from("Alpha"),
        Arc::<str>::from("Beta"),
        Arc::<str>::from("Gamma"),
    ];
    let changed = Rc::new(Cell::new(false));
    let picked_index = Rc::new(Cell::new(None::<usize>));
    let picked = Rc::new(RefCell::new(None::<Arc<str>>));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  changed_out: &Rc<Cell<bool>>,
                  picked_index_out: &Rc<Cell<Option<usize>>>,
                  picked_out: &Rc<RefCell<Option<Arc<str>>>>| {
        crate::imui_raw(cx, |ui| {
            let response = ui.input_text_completion_model_with_options(
                "imui-input-text-completion-picker-keyboard.popup",
                &model,
                &candidates,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from(
                            "imui-input-text-completion-picker-keyboard.input",
                        )),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-input-text-completion-picker-keyboard")),
                    ..Default::default()
                },
            );
            changed_out.set(response.changed());
            picked_index_out.set(response.picked_index());
            picked_out.replace(response.picked.clone());
        })
    };

    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-keyboard",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );

    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-keyboard.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);
    text_input_event(&mut ui, &mut app, &mut services, "a");

    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-keyboard",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-keyboard.option.1",
    ));
    assert!(!picker_option_selected(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-keyboard.option.0",
    ));
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );
    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-keyboard",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(picker_option_selected(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-keyboard.option.0",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );
    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-keyboard",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );
    assert!(picker_option_selected(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-keyboard.option.1",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Enter,
        Modifiers::default(),
    );
    app.advance_frame();
    let changed_out = changed.clone();
    let picked_index_out = picked_index.clone();
    let picked_out = picked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-keyboard",
        |cx| render(cx, &changed_out, &picked_index_out, &picked_out),
    );

    assert!(changed.get());
    assert_eq!(picked_index.get(), Some(1));
    assert_eq!(picked.borrow().as_deref(), Some("Beta"));
    assert_eq!(app.models().get_cloned(&model).as_deref(), Some("Beta"));
}

#[test]
fn input_text_completion_picker_keyboard_navigation_exposes_active_descendant_semantics() {
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
    let candidates = vec![
        Arc::<str>::from("Alpha"),
        Arc::<str>::from("Beta"),
        Arc::<str>::from("Gamma"),
    ];

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let _ = ui.input_text_completion_model_with_options(
                "imui-input-text-completion-picker-a11y.popup",
                &model,
                &candidates,
                InputTextPickerOptions {
                    input: InputTextOptions {
                        test_id: Some(Arc::from("imui-input-text-completion-picker-a11y.input")),
                        ..Default::default()
                    },
                    test_id: Some(Arc::from("imui-input-text-completion-picker-a11y")),
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
        "imui-input-text-completion-picker-a11y",
        render,
    );
    let input = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-input-text-completion-picker-a11y.input",
    );
    click_at(&mut ui, &mut app, &mut services, input);
    text_input_event(&mut ui, &mut app, &mut services, "a");

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-a11y",
        render,
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
        "imui-input-text-completion-picker-a11y",
        render,
    );
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-input-text-completion-picker-a11y",
        render,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let input_node = snap
        .nodes
        .iter()
        .find(|node| {
            node.test_id.as_deref() == Some("imui-input-text-completion-picker-a11y.input")
        })
        .expect("expected picker input semantics node");
    let active_option = snap
        .nodes
        .iter()
        .find(|node| {
            node.test_id.as_deref() == Some("imui-input-text-completion-picker-a11y.option.0")
        })
        .expect("expected active picker option semantics node");
    let popup_panel = snap
        .nodes
        .iter()
        .find(|node| {
            node.test_id.as_deref()
                == Some("imui-popup-imui-input-text-completion-picker-a11y.popup")
        })
        .expect("expected picker popup panel semantics node");

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
            picked_out.replace(response.picked.clone());
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
    assert!(picker_option_selected(
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
    assert!(picker_option_selected(
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
