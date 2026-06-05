use super::*;

#[test]
fn context_menu_popup_item_pointer_move_then_click_still_works_after_extra_frames() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  open_out: &Rc<Cell<bool>>,
                  clicked_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            let resp = ui.button("OK");
            open_out.set(ui.begin_popup_context_menu_with_options(
                "ctx",
                resp,
                PopupMenuOptions {
                    estimated_size: Size::new(Px(120.0), Px(60.0)),
                    ..Default::default()
                },
                |ui| {
                    let toggle = ui.menu_item_with_options(
                        "Toggle",
                        MenuItemOptions {
                            test_id: Some(Arc::from("imui-popup-ctx-item-toggle-move-delayed")),
                            ..Default::default()
                        },
                    );
                    clicked_out.set(toggle.clicked());
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
        "imui-popup-context-menu-item-clicked-move-delayed",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(!open.get());
    assert!(!clicked.get());

    let at = first_child_point(&ui, root);
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-clicked-move-delayed",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(!clicked.get());

    for _ in 0..2 {
        app.advance_frame();
        ui.request_semantics_snapshot();
        let open_out = open.clone();
        let clicked_out = clicked.clone();
        let _root = run_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "imui-popup-context-menu-item-clicked-move-delayed",
            |cx| render(cx, &open_out, &clicked_out),
        );
        assert!(
            open.get(),
            "expected popup to remain open across extra frames"
        );
        assert!(!clicked.get());
    }

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-toggle-move-delayed");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
    );
    pointer_move_at(
        &mut ui,
        &mut app,
        &mut services,
        click_point,
        MouseButtons::default(),
    );
    click_at(&mut ui, &mut app, &mut services, click_point);

    app.advance_frame();
    let open_out = open.clone();
    let clicked_out = clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-clicked-move-delayed",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(
        clicked.get(),
        "expected delayed pointer move + click to keep working for popup items"
    );
}
