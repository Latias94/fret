use super::*;

#[test]
fn begin_submenu_helper_hover_switches_sibling_after_open_delay() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let recent_open = Rc::new(Cell::new(false));
    let history_open = Rc::new(Cell::new(false));
    let history_hovered = Rc::new(Cell::new(false));
    let history_hovered_raw = Rc::new(Cell::new(false));
    let history_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let recent_open = recent_open.clone();
        let history_open = history_open.clone();
        let history_hovered = history_hovered.clone();
        let history_hovered_raw = history_hovered_raw.clone();
        let history_hovered_raw_below_barrier = history_hovered_raw_below_barrier.clone();
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-sibling-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-sibling-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-sibling-switch.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-sibling-switch.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            recent_open.set(recent.open());
                            let history = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-sibling-switch.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-sibling-switch.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            history_open.set(history.open());
                            history_hovered.set(history.response().hovered());
                            history_hovered_raw.set(history.response().pointer_hovered_raw());
                            history_hovered_raw_below_barrier
                                .set(history.response().pointer_hovered_raw_below_barrier());
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
        "imui-submenu-sibling-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.recent",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        recent_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected submenu open timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.history.yesterday",
    ));

    let history_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-sibling-switch.file.history",
    );
    let clear_grace_point = Point::new(
        Px((history_trigger.x.0 - 120.0).max(4.0)),
        history_trigger.y,
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        clear_grace_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        history_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected sibling hover to dispatch a timer");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-sibling-switch",
        &render,
    );

    assert!(
        !has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-submenu-sibling-switch.file.recent.project",
        ),
        "expected the previous submenu to close after sibling hover-switch (recent_open={} history_open={} history_hovered={} history_hovered_raw={} history_hovered_raw_below_barrier={})",
        recent_open.get(),
        history_open.get(),
        history_hovered.get(),
        history_hovered_raw.get(),
        history_hovered_raw_below_barrier.get()
    );
    assert!(
        has_test_id(
            &mut ui,
            &mut app,
            &mut services,
            bounds,
            "imui-submenu-sibling-switch.file.history.yesterday",
        ),
        "expected the sibling submenu to open after hover-switch (recent_open={} history_open={} history_hovered={} history_hovered_raw={} history_hovered_raw_below_barrier={})",
        recent_open.get(),
        history_open.get(),
        history_hovered.get(),
        history_hovered_raw.get(),
        history_hovered_raw_below_barrier.get()
    );
}
