use super::*;

#[test]
fn combo_model_popup_escape_closes_and_restores_trigger_focus() {
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

    let model = app.models_mut().insert(None::<Arc<str>>);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);
    let focus_before_open = ui.focus();
    assert!(focus_before_open.is_some());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape.option.0",
    ));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-escape",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-escape-popup",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-escape")),
                        ..Default::default()
                    },
                );
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-escape.option.0",
    ));
    assert_eq!(ui.focus(), focus_before_open);
}

#[test]
fn combo_model_popup_scope_override_controls_popup_test_id() {
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

    let model = app.models_mut().insert(None::<Arc<str>>);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-popup-scope-override",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-scope",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-scope",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-popup-scope-override",
                    "Mode",
                    &model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-scope")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-select-popup-scope-override",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-imui-select-popup-imui-select-scope",
    ));
}
