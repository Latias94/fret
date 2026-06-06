use super::*;

#[test]
fn menu_item_activate_shortcut_repeat_is_opt_in() {
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
    let default_clicks = Rc::new(Cell::new(0u32));
    let repeat_clicks = Rc::new(Cell::new(0u32));
    let default_shortcut = ctrl_shortcut(KeyCode::KeyJ);
    let repeat_shortcut = ctrl_shortcut(KeyCode::KeyK);

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  open_out: &Rc<Cell<bool>>,
                  default_clicks_out: &Rc<Cell<u32>>,
                  repeat_clicks_out: &Rc<Cell<u32>>| {
        crate::imui_raw(cx, |ui| {
            let resp = ui.button("OK");
            open_out.set(ui.begin_popup_context_menu_with_options(
                "ctx-repeat",
                resp,
                PopupMenuOptions {
                    estimated_size: Size::new(Px(160.0), Px(90.0)),
                    ..Default::default()
                },
                |ui| {
                    let default_item = ui.menu_item_with_options(
                        "Item A",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-popup-shortcut-repeat.default")),
                            activate_shortcut: Some(default_shortcut),
                            ..Default::default()
                        },
                    );
                    if default_item.clicked() {
                        default_clicks_out.set(default_clicks_out.get() + 1);
                    }

                    let repeat_item = ui.menu_item_with_options(
                        "Item B",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-popup-shortcut-repeat.repeat")),
                            activate_shortcut: Some(repeat_shortcut),
                            shortcut_repeat: true,
                            ..Default::default()
                        },
                    );
                    if repeat_item.clicked() {
                        repeat_clicks_out.set(repeat_clicks_out.get() + 1);
                    }
                },
            ));
        })
    };

    let open_out = open.clone();
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        |cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert!(!open.get());
    assert_eq!(default_clicks.get(), 0);
    assert_eq!(repeat_clicks.get(), 0);

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
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        &|cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert!(open.get());
    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-shortcut-repeat.default",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let open_out = open.clone();
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        &|cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert_eq!(default_clicks.get(), 1);
    assert_eq!(repeat_clicks.get(), 0);

    let _default_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-shortcut-repeat.default",
    );
    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyJ);

    let open_out = open.clone();
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        &|cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert_eq!(
        default_clicks.get(),
        1,
        "expected repeated keydown to be ignored unless shortcut_repeat is enabled"
    );

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-shortcut-repeat.repeat",
    );

    key_down_ctrl(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let open_out = open.clone();
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        &|cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert_eq!(repeat_clicks.get(), 1);

    let _repeat_node = focus_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-popup-shortcut-repeat.repeat",
    );
    key_down_ctrl_repeat(&mut ui, &mut app, &mut services, KeyCode::KeyK);

    let open_out = open.clone();
    let default_clicks_out = default_clicks.clone();
    let repeat_clicks_out = repeat_clicks.clone();
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-shortcut-repeat",
        &|cx| render(cx, &open_out, &default_clicks_out, &repeat_clicks_out),
    );
    assert_eq!(
        repeat_clicks.get(),
        2,
        "expected repeated keydown to retrigger only when shortcut_repeat is enabled"
    );
}
