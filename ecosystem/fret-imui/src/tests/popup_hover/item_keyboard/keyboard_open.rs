use super::*;

#[test]
fn context_menu_popup_keyboard_open_focuses_first_item_and_escape_restores_trigger_focus() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(240.0), Px(120.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let open = Rc::new(Cell::new(false));
    let open_out = open.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(!open.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    let focus_before_open = ui.focus();
    assert!(
        focus_before_open.is_some(),
        "expected trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_with_options(
                            "Item A",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-a")),
                                ..Default::default()
                            },
                        );
                        ui.menu_item_with_options(
                            "Item B",
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-b")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-ctx-item-a"));

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::Escape,
        Modifiers::default(),
    );

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-keyboard-open",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
            })
        },
    );
    assert!(!open.get());
    assert_eq!(ui.focus(), focus_before_open);
}
