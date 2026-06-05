use super::*;

#[test]
fn begin_menu_helper_toggles_popup_and_closes_after_command_activate() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.begin-menu.open");
    app.commands_mut().register(
        command.clone(),
        CommandMeta::new("Open File").with_default_keybindings([DefaultKeybinding::single(
            PlatformFilter::All,
            KeyChord::new(
                KeyCode::KeyO,
                Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
            ),
        )]),
    );

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.menu_bar_with_options(
                fret_ui_kit::imui::MenuBarOptions {
                    test_id: Some(Arc::from("imui-begin-menu.root")),
                    ..Default::default()
                },
                |ui| {
                    let _ = ui.begin_menu_with_options(
                        "file",
                        "File",
                        fret_ui_kit::imui::BeginMenuOptions {
                            test_id: Some(Arc::from("imui-begin-menu.file")),
                            ..Default::default()
                        },
                        |ui| {
                            let open = ui.popup_open_model("file");
                            ui.menu_item_command_with_options(
                                command.clone(),
                                MenuItemOptions {
                                    close_popup: Some(open),
                                    test_id: Some(Arc::from("imui-begin-menu.file.open")),
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
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let trigger = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file",
    );
    click_at(&mut ui, &mut app, &mut services, trigger);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    );
    click_at(&mut ui, &mut app, &mut services, item);
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
        "imui-begin-menu",
        |cx| render(cx, &command),
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-begin-menu.file.open",
    ));
}
