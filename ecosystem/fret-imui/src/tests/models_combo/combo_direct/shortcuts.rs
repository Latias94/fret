use super::*;

#[test]
fn combo_activate_shortcut_is_scoped_to_focused_trigger() {
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
    let target_open = Rc::new(Cell::new(false));
    let other_open = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  target_open_out: &Rc<Cell<bool>>,
                  other_open_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let target = ui.combo_with_options(
                    "imui-combo-shortcut-target-popup",
                    "Target",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-shortcut.target")),
                        activate_shortcut: Some(shortcut),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-shortcut.target.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let other = ui.combo_with_options(
                    "imui-combo-shortcut-other-popup",
                    "Other",
                    "Beta",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-shortcut.other")),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Beta",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-shortcut.other.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                target_open_out.set(target.open());
                other_open_out.set(other.open());
            });
        })
    };

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        |cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(!target_open.get());
    assert!(!other_open.get());

    let _other_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.other",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        &|cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(!target_open.get());
    assert!(!other_open.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target.option.0",
    ));

    let _target_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let target_open_out = target_open.clone();
    let other_open_out = other_open.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-shortcut",
        &|cx| render(cx, &target_open_out, &other_open_out),
    );
    assert!(target_open.get());
    assert!(!other_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-shortcut.target.option.0",
    ));
}

#[test]
fn combo_activate_shortcut_repeat_is_opt_in() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.vertical(|ui| {
                let _ = ui.combo_with_options(
                    "imui-combo-repeat-default-popup",
                    "Default",
                    "Alpha",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-repeat.default")),
                        activate_shortcut: Some(default_shortcut),
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Alpha",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-repeat.default.option.0")),
                                ..Default::default()
                            },
                        );
                    },
                );
                let _ = ui.combo_with_options(
                    "imui-combo-repeat-enabled-popup",
                    "Repeat",
                    "Beta",
                    ComboOptions {
                        test_id: Some(Arc::from("imui-combo-repeat.repeat")),
                        activate_shortcut: Some(repeat_shortcut),
                        shortcut_repeat: true,
                        ..Default::default()
                    },
                    |ui| {
                        let _ = ui.selectable_with_options(
                            "Beta",
                            SelectableOptions {
                                test_id: Some(Arc::from("imui-combo-repeat.repeat.option.0")),
                                ..Default::default()
                            },
                        );
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
        "imui-combo-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default.option.0",
    ));
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-combo-repeat.default.option.0",
        ),
        "expected repeated keydown to leave default combo trigger open"
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
        "imui-combo-repeat",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.default.option.0",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat.option.0",
    ));
    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-combo-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-combo-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-combo-repeat.repeat.option.0",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
