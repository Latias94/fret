use super::*;

#[test]
fn begin_menu_activate_shortcut_repeat_is_opt_in() {
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

    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-repeat.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file-default",
                        "Default",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-repeat.default")),
                            activate_shortcut: Some(default_shortcut),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-repeat.default.item")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "file-repeat",
                        "Repeat",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-repeat.repeat")),
                            activate_shortcut: Some(repeat_shortcut),
                            shortcut_repeat: true,
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-begin-menu-repeat.repeat.item")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
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
        "imui-begin-menu-repeat",
        render,
    );

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default.item",
    ));

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.default",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-menu-repeat.default.item",
        ),
        "expected repeated keydown to leave default shortcut trigger open"
    );

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat.item",
    ));

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-repeat.repeat",
    );

    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-repeat",
        &render,
    );
    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-begin-menu-repeat.repeat.item",
        ),
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
