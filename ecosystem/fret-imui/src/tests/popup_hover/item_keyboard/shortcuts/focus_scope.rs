use super::*;

#[test]
fn menu_item_activate_shortcut_is_scoped_to_focused_popup_item_and_preserves_arrow_nav() {
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
    let clicked = Rc::new(Cell::new(false));
    let shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  open_out: &Rc<Cell<bool>>,
                  clicked_out: &Rc<Cell<bool>>| {
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
                    let _a = ui.menu_item_with_options(
                        "Item A",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-popup-shortcut-item-a")),
                            ..Default::default()
                        },
                    );
                    let b = ui.menu_item_with_options(
                        "Item B",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-popup-shortcut-item-b")),
                            activate_shortcut: Some(shortcut),
                            ..Default::default()
                        },
                    );
                    clicked_out.set(b.clicked());
                },
            ));
        })
    };

    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(!open.get());
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    click_at(&mut ui, &mut app, &mut services, at);
    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut",
        &|cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(!clicked.get());

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut",
        &|cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(!clicked.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ArrowDown,
        Modifiers::default(),
    );

    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut",
        &|cx| render(cx, &open_out, &clicked_out),
    );
    let focus = ui.focus().expect("focus");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let focused_test_id = snap
        .nodes
        .iter()
        .find(|n| n.id == focus)
        .and_then(|n| n.test_id.as_deref());
    assert_eq!(focused_test_id, Some("imui-popup-shortcut-item-b"));
    assert!(open.get());
    assert!(!clicked.get());

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut",
        &|cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(clicked.get());
}
