use super::*;

#[test]
fn menu_item_command_uses_command_metadata_shortcut_and_gating() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.open-file");
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
            ui.menu_item_command_with_options(
                command.clone(),
                MenuItemOptions {
                    test_id: Some(Arc::from("imui-menu-command")),
                    ..Default::default()
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
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-menu-command"))
        .expect("menu item semantics node");
    assert_eq!(node.label.as_deref(), Some("Open File"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command.shortcut",
    ));

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-menu-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}
#[test]
fn button_command_uses_command_metadata_and_gating() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let command = CommandId::from("test.open-project");
    app.commands_mut()
        .register(command.clone(), CommandMeta::new("Open Project"));

    let render = |cx: &mut ElementContext<'_, TestHost>, command: &CommandId| {
        crate::imui_raw(cx, |ui| {
            ui.button_command_with_options(
                command.clone(),
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-button-command")),
                    ..Default::default()
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
        "imui-button-command",
        |cx| render(cx, &command),
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|node| node.test_id.as_deref() == Some("imui-button-command"))
        .expect("button semantics node");
    assert_eq!(node.label.as_deref(), Some("Open Project"));
    assert_eq!(node.role, fret_core::SemanticsRole::Button);

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);
    assert!(app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));

    app.effects.clear();
    app.set_global(WindowCommandEnabledService::default());
    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.set_enabled(window, command.clone(), false);
    });
    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-button-command",
        |cx| render(cx, &command),
    );

    let disabled_click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-button-command",
    );
    click_at(&mut ui, &mut app, &mut services, disabled_click_point);
    assert!(!app.effects.iter().any(|effect| {
        matches!(
            effect,
            Effect::Command { window: Some(target_window), command: target_command }
                if *target_window == window && *target_command == command
        )
    }));
}
