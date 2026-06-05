use super::*;

#[test]
fn menu_and_submenu_helpers_report_toggle_and_trigger_edges() {
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

    let menu_open = Rc::new(Cell::new(false));
    let menu_opened = Rc::new(Cell::new(false));
    let menu_closed = Rc::new(Cell::new(false));
    let menu_activated = Rc::new(Cell::new(false));
    let menu_deactivated = Rc::new(Cell::new(false));
    let submenu_open = Rc::new(Cell::new(false));
    let submenu_opened = Rc::new(Cell::new(false));
    let submenu_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let menu_open = menu_open.clone();
        let menu_opened = menu_opened.clone();
        let menu_closed = menu_closed.clone();
        let menu_activated = menu_activated.clone();
        let menu_deactivated = menu_deactivated.clone();
        let submenu_open = submenu_open.clone();
        let submenu_opened = submenu_opened.clone();
        let submenu_clicked = submenu_clicked.clone();

        crate::imui_raw(cx, move |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-menu-response.root")),
                    ..Default::default()
                },
                |ui| {
                    let menu = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-response.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let submenu = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-menu-response.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-menu-response.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            submenu_open.set(submenu.open());
                            submenu_opened.set(submenu.opened());
                            submenu_clicked.set(submenu.clicked());
                        },
                    );
                    menu_open.set(menu.open());
                    menu_opened.set(menu.opened());
                    menu_closed.set(menu.closed());
                    menu_activated.set(menu.response().activated());
                    menu_deactivated.set(menu.response().deactivated());
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
        "imui-menu-response",
        render,
    );
    assert!(!menu_open.get());
    assert!(!menu_opened.get());
    assert!(!menu_closed.get());
    assert!(!menu_activated.get());
    assert!(!menu_deactivated.get());
    assert!(!submenu_open.get());
    assert!(!submenu_opened.get());
    assert!(!submenu_clicked.get());

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file",
    );
    pointer_down_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(!menu_open.get());
    assert!(menu_activated.get());
    assert!(!menu_deactivated.get());

    pointer_up_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(menu_open.get());
    assert!(menu_opened.get());
    assert!(menu_deactivated.get());
    assert!(!submenu_open.get());

    let submenu_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file.recent",
    );
    click_at(&mut ui, &mut app, &mut services, submenu_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-response",
        &render,
    );
    assert!(menu_open.get());
    assert!(submenu_open.get());
    assert!(submenu_opened.get());
    assert!(submenu_clicked.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-response.file.recent.project",
    ));
}
