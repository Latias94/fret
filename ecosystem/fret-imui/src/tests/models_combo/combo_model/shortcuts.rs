use super::*;

#[test]
fn combo_model_activate_shortcut_is_scoped_to_focused_trigger() {
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

    let shortcut = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let target_model = app.models_mut().insert(None::<Arc<str>>);
    let other_model = app.models_mut().insert(None::<Arc<str>>);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );

    let _other_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        &|cx| {
            crate::imui_raw(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target.option.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other.option.0",
    ));

    let _target_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-shortcut",
        &|cx| {
            crate::imui_raw(cx, |ui| {
                ui.vertical(|ui| {
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-target-popup",
                        "Target",
                        &target_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.target")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    let _ = ui.combo_model_with_options(
                        "imui-select-shortcut-other-popup",
                        "Other",
                        &other_model,
                        &items,
                        ComboModelOptions {
                            test_id: Some(Arc::from("imui-select-shortcut.other")),
                            ..Default::default()
                        },
                    );
                });
            })
        },
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.target.option.0",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-shortcut.other.option.0",
    ));
}

#[test]
fn combo_model_activate_shortcut_repeat_is_opt_in() {
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

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);
    let items = vec![Arc::<str>::from("Alpha"), Arc::<str>::from("Beta")];
    let default_model = app.models_mut().insert(None::<Arc<str>>);
    let repeat_model = app.models_mut().insert(None::<Arc<str>>);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_model_with_options(
                    "imui-select-repeat-default-popup",
                    "Default",
                    &default_model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-repeat.default")),
                        activate_shortcut: Some(default_shortcut),
                        ..Default::default()
                    },
                );
                let _ = ui.combo_model_with_options(
                    "imui-select-repeat-enabled-popup",
                    "Repeat",
                    &repeat_model,
                    &items,
                    ComboModelOptions {
                        test_id: Some(Arc::from("imui-select-repeat.repeat")),
                        activate_shortcut: Some(repeat_shortcut),
                        shortcut_repeat: true,
                        ..Default::default()
                    },
                );
            });
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default.option.0",
    ));
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-select-repeat.default.option.0",
        ),
        "expected repeated keydown to leave default combo-model trigger open"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.default.option.0",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat.option.0",
    ));
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-select-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-select-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-select-repeat.repeat.option.0",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
