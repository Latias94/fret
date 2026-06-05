use super::*;

#[test]
fn child_region_helper_can_host_menu_bar_and_popup_menu() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.child_region_with_options(
                "imui-child-region-with-menu",
                ChildRegionOptions {
                    test_id: Some(Arc::from("imui-child-region-with-menu")),
                    content_test_id: Some(Arc::from("imui-child-region-with-menu.content")),
                    ..Default::default()
                },
                |ui| {
                    ui.menu_bar_with_options(
                        fret_ui_kit::imui::MenuBarOptions {
                            test_id: Some(Arc::from("imui-child-region-with-menu.menubar")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_menu_with_options(
                                "file",
                                "File",
                                fret_ui_kit::imui::BeginMenuOptions {
                                    test_id: Some(Arc::from("imui-child-region-with-menu.file")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Open",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-child-region-with-menu.file.open",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                    ui.menu_item_with_options(
                        "Body row",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-child-region-with-menu.body")),
                            ..Default::default()
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
        "imui-child-region-with-menu",
        render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.content",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.menubar",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.body",
    ));

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-child-region-with-menu",
        &render,
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-child-region-with-menu.file.open",
    ));
}
