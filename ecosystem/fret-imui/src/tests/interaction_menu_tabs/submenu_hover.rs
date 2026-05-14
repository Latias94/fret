use super::*;

#[test]
fn begin_submenu_helper_opens_nested_menu_and_tracks_expanded_semantics() {
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

    let command = CommandId::from("test.begin-submenu.open-recent");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Recent Project").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyR,
                Modifiers {
                    ctrl: true,
                    shift: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let file_open = Rc::new(Cell::new(false));
    let recent_open = Rc::new(Cell::new(false));
    let file_open_out = file_open.clone();
    let recent_open_out = recent_open.clone();
    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-submenu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-submenu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let recent_open = ui.popup_open_model("recent");
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from("imui-begin-submenu.file.recent")),
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.menu_item_command_with_options(
                                        command.clone(),
                                        MenuItemOptions {
                                            close_popup: Some(recent_open),
                                            test_id: Some(Arc::from(
                                                "imui-begin-submenu.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                        },
                    );
                },
            );

            let file_popup = ui.popup_open_model("file");
            let recent_popup = ui.popup_open_model("recent");
            file_open_out.set(
                ui.cx_mut()
                    .read_model(&file_popup, fret_ui::Invalidation::Paint, |_app, value| {
                        *value
                    })
                    .unwrap_or(false),
            );
            recent_open_out.set(
                ui.cx_mut()
                    .read_model(
                        &recent_popup,
                        fret_ui::Invalidation::Paint,
                        |_app, value| *value,
                    )
                    .unwrap_or(false),
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!file_open.get());
    assert!(!recent_open.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(!recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    ));

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert_eq!(recent_node.role, fret_core::SemanticsRole::MenuItem);
    assert!(!recent_node.flags.expanded);

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent",
    );
    click_at(&mut ui, &mut app, &mut services, recent_trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(file_open.get());
    assert!(recent_open.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(
        file_open.get(),
        "expected parent menu to remain open (file_open={} recent_open={})",
        file_open.get(),
        recent_open.get()
    );
    assert!(
        recent_open.get(),
        "expected submenu to remain open (file_open={} recent_open={})",
        file_open.get(),
        recent_open.get()
    );
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let recent_node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-begin-submenu.file.recent"))
        .expect("recent submenu semantics node");
    assert!(recent_node.flags.expanded);

    let project_item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    );
    click_at(&mut ui, &mut app, &mut services, project_item);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-submenu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-submenu.file.recent.project",
    ));
}
#[test]
fn begin_menu_helper_hover_switches_top_level_popup_after_trigger_hover_delay() {
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
                    test_id: Some(Arc::from("imui-menu-hover-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-hover-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Open",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-menu-hover-switch.file.open")),
                                    ..Default::default()
                                },
                            );
                        },
                    );
                    let _ = ui.begin_menu_with_options(
                        "edit",
                        "Edit",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-menu-hover-switch.edit")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.menu_item_with_options(
                                "Copy",
                                MenuItemOptions {
                                    test_id: Some(Arc::from("imui-menu-hover-switch.edit.copy")),
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
        "imui-menu-hover-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file.open",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit.copy",
    ));

    app.effects.clear();
    let edit_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit",
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        edit_trigger,
        MouseButtons::default(),
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    let dispatched = dispatch_all_timers(&mut ui, &mut app, &mut services);
    assert!(dispatched > 0, "expected hover-switch timer to arm");

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-hover-switch",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.file.open",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-hover-switch.edit.copy",
    ));
}
#[test]
fn begin_submenu_helper_hover_opens_submenu_after_pointer_entry() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-hover-switch.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-hover-switch.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-hover-switch.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-hover-switch.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-hover-switch.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-hover-switch.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
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
        "imui-submenu-hover-switch",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.history",
    ));

    app.effects.clear();
    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent",
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
        "imui-submenu-hover-switch",
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
        "imui-submenu-hover-switch",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-hover-switch.file.history.yesterday",
    ));
}
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

#[test]
fn begin_submenu_helper_defers_sibling_switch_inside_grace_corridor() {
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
    let history_hovered = Rc::new(Cell::new(false));
    let history_hovered_raw = Rc::new(Cell::new(false));
    let history_hovered_raw_below_barrier = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        let history_hovered = history_hovered.clone();
        let history_hovered_raw = history_hovered_raw.clone();
        let history_hovered_raw_below_barrier = history_hovered_raw_below_barrier.clone();
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-grace-corridor.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-grace-corridor.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let history = ui.begin_submenu_with_options(
                                "history",
                                "History",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-grace-corridor.file.history",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Yesterday",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-grace-corridor.file.history.yesterday",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
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
        "imui-submenu-grace-corridor",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent",
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
        "imui-submenu-grace-corridor",
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
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let recent_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.recent");
    let history_bounds = bounds_for_test_id(&ui, "imui-submenu-grace-corridor.file.history");
    let recent_popup_bounds = bounds_for_test_id(&ui, "imui-popup-recent");
    let (grace_exit_point, history_grace_point) =
        find_grace_corridor_transition_points(recent_bounds, history_bounds, recent_popup_bounds)
            .expect("expected a history point inside the submenu grace corridor");

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        grace_exit_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        history_grace_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );

    assert!(
        history_hovered.get()
            || history_hovered_raw.get()
            || history_hovered_raw_below_barrier.get(),
        "expected pointer to hit the sibling trigger inside grace corridor (exit={grace_exit_point:?} hit={history_grace_point:?})"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));

    let grace_timeout =
        fret_ui_kit::primitives::menu::sub::MenuSubmenuConfig::default().pointer_grace_timeout;
    let pending = pending_nonrepeating_timer_tokens_after(&app, grace_timeout);
    let dispatched = dispatch_timer_tokens(&mut ui, &mut app, &mut services, &pending);
    assert!(
        dispatched > 0,
        "expected pointer grace timeout timer to be present"
    );

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-grace-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.recent.project",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-grace-corridor.file.history.yesterday",
    ));
}

#[test]
fn begin_submenu_helper_safe_corridor_cancels_close_timer() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-submenu-safe-corridor.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-submenu-safe-corridor.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let _ = ui.begin_submenu_with_options(
                                "recent",
                                "Recent",
                                fret_ui_kit::imui::BeginSubmenuOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-safe-corridor.file.recent",
                                    )),
                                    ..Default::default()
                                },
                                |ui| {
                                    let _ = ui.menu_item_with_options(
                                        "Project",
                                        MenuItemOptions {
                                            test_id: Some(Arc::from(
                                                "imui-submenu-safe-corridor.file.recent.project",
                                            )),
                                            ..Default::default()
                                        },
                                    );
                                },
                            );
                            let _ = ui.menu_item_with_options(
                                "Other",
                                MenuItemOptions {
                                    test_id: Some(Arc::from(
                                        "imui-submenu-safe-corridor.file.other",
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
        "imui-submenu-safe-corridor",
        render,
    );

    let file_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file",
    );
    click_at(&mut ui, &mut app, &mut services, file_trigger);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    let recent_trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent",
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
        "imui-submenu-safe-corridor",
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
        "imui-submenu-safe-corridor",
        &render,
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent.project",
    ));

    let submenu_cfg = fret_ui_kit::primitives::menu::sub::MenuSubmenuConfig::default();
    let recent_bounds = bounds_for_test_id(&ui, "imui-submenu-safe-corridor.file.recent");
    let recent_popup_bounds = bounds_for_test_id(&ui, "imui-popup-recent");
    let (unsafe_point, safe_point) = find_safe_hover_corridor_points(
        bounds,
        recent_bounds,
        recent_popup_bounds,
        submenu_cfg.safe_hover_buffer,
    )
    .expect("expected safe/unsafe corridor points around the open submenu");

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        unsafe_point,
        MouseButtons::default(),
    );
    let close_tokens = pending_nonrepeating_timer_tokens_after(&app, submenu_cfg.close_delay);
    assert!(
        !close_tokens.is_empty(),
        "expected unsafe pointer move to arm a close-delay timer (unsafe_point={unsafe_point:?})"
    );
    let close_token = close_tokens[0];

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    app.effects.clear();
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        safe_point,
        MouseButtons::default(),
    );
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-submenu-safe-corridor",
        &render,
    );

    assert!(
        app.effects
            .iter()
            .any(|effect| matches!(effect, Effect::CancelTimer { token } if *token == close_token)),
        "expected safe corridor pointer move to cancel the close-delay timer (safe_point={safe_point:?} close_token={close_token:?} effects={:?})",
        app.effects
    );
    assert!(
        pending_nonrepeating_timer_tokens_after(&app, submenu_cfg.close_delay).is_empty(),
        "expected safe corridor pointer move to avoid arming a new close-delay timer"
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-submenu-safe-corridor.file.recent.project",
    ));
}
