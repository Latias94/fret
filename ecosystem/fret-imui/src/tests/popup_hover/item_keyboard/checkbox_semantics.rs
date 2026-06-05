use super::*;

#[test]
fn menu_item_checkbox_stamps_semantics_checked_state() {
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

    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-checkbox-semantics",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_checkbox_with_options(
                            "Flag",
                            true,
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
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
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    ui.request_semantics_snapshot();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-menu-item-checkbox-semantics",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                let open_out = open.clone();
                open_out.set(ui.begin_popup_context_menu_with_options(
                    "ctx",
                    resp,
                    PopupMenuOptions {
                        estimated_size: Size::new(Px(160.0), Px(90.0)),
                        ..Default::default()
                    },
                    |ui| {
                        ui.menu_item_checkbox_with_options(
                            "Flag",
                            true,
                            MenuItemOptions {
                                test_id: Some(Arc::from("imui-popup-ctx-item-flag")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let node = snap
        .nodes
        .iter()
        .find(|n| n.test_id.as_deref() == Some("imui-popup-ctx-item-flag"))
        .expect("checkbox node");
    assert_eq!(node.role, SemanticsRole::MenuItemCheckbox);
    assert_eq!(node.flags.checked, Some(true));
}
