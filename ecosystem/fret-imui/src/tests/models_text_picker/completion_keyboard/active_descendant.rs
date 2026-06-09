use super::*;

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
