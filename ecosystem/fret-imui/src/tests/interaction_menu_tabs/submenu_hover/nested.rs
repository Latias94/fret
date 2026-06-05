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
