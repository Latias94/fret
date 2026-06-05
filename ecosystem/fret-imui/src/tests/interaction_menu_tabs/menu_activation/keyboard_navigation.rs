use super::*;

#[test]
fn begin_menu_arrow_down_opens_menu_and_focuses_first_item() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-arrow-open.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-open.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-open.file.open",
                                    )),
                                    ..Default::default()
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Save",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-open.file.save",
                                    )),
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
        "imui-begin-menu-arrow-open",
        render,
    );

    let _file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-open.file",
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-open",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-open.file.open",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-open.file.open"))
    );
}

#[test]
fn begin_menu_horizontal_arrows_switch_active_top_level_menu() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(460.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu-arrow-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-switch.file.open",
                                    )),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "edit",
                        "Edit",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu-arrow-switch.edit")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-begin-menu-arrow-switch.edit.copy",
                                    )),
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
        "imui-begin-menu-arrow-switch",
        render,
    );

    let _file_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file",
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowRight,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.edit.copy"))
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowLeft,
        Modifiers::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu-arrow-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu-arrow-switch.edit.copy",
    ));
    assert_eq!(
        current_focus_test_id(&mut ui, &mut app, &mut services, bounds),
        Some(String::from("imui-begin-menu-arrow-switch.file.open"))
    );
}
