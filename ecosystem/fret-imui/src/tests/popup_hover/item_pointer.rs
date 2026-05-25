use super::*;

fn hit_or_ancestor_test_id<'a>(
    snap: &'a fret_core::SemanticsSnapshot,
    mut id: fret_core::NodeId,
) -> Option<&'a str> {
    loop {
        let node = snap.nodes.iter().find(|n| n.id == id)?;
        if let Some(test_id) = node.test_id.as_deref() {
            return Some(test_id);
        }
        id = node.parent?;
    }
}

#[test]
fn context_menu_popup_item_click_closes_popup() {
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
        "imui-popup-context-menu-item-close",
        |cx| {
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
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
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
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    ui.request_semantics_snapshot();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
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
                        let open_model = ui.popup_open_model("ctx");
                        ui.menu_item_with_options(
                            "Close",
                            MenuItemOptions {
                                close_popup: Some(open_model),
                                test_id: Some(Arc::from("imui-popup-ctx-item-close")),
                                ..Default::default()
                            },
                        );
                    },
                ));
            })
        },
    );
    assert!(open.get());

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-close");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
    );
    let hit = ui.debug_hit_test(click_point).hit.expect("hit node");
    let snap = ui.semantics_snapshot().expect("semantics snapshot");
    let hit_test_id = hit_or_ancestor_test_id(snap, hit);
    assert_eq!(
        hit_test_id,
        Some("imui-popup-ctx-item-close"),
        "expected click to hit the menu item pressable or one of its descendants"
    );

    click_at(&mut ui, &mut app, &mut services, click_point);

    app.advance_frame();
    let open_out = open.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-popup-context-menu-item-close",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let resp = ui.button("OK");
                open_out.set(ui.begin_popup_context_menu("ctx", resp, |_ui| {}));
            })
        },
    );
    assert!(!open.get());
}

#[test]
fn context_menu_popup_item_pointer_click_reports_clicked() {
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
                            test_id: Some(Arc::from("imui-popup-ctx-item-toggle")),
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
        "imui-popup-context-menu-item-clicked",
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
        "imui-popup-context-menu-item-clicked",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(!clicked.get());

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-toggle");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
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
        "imui-popup-context-menu-item-clicked",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(
        clicked.get(),
        "expected pointer click to set menu item clicked()"
    );
}

#[test]
fn context_menu_popup_item_pointer_click_still_works_after_extra_frames() {
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
                            test_id: Some(Arc::from("imui-popup-ctx-item-toggle-delayed")),
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
        "imui-popup-context-menu-item-clicked-delayed",
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
        "imui-popup-context-menu-item-clicked-delayed",
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
            "imui-popup-context-menu-item-clicked-delayed",
            |cx| render(cx, &open_out, &clicked_out),
        );
        assert!(
            open.get(),
            "expected popup to remain open across extra frames"
        );
        assert!(!clicked.get());
    }

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-toggle-delayed");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
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
        "imui-popup-context-menu-item-clicked-delayed",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(
        clicked.get(),
        "expected delayed pointer click to keep working for popup items"
    );
}

#[test]
fn context_menu_popup_item_pointer_click_still_works_after_idle_frames_without_render() {
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
                            test_id: Some(Arc::from("imui-popup-ctx-item-toggle-idle-frames")),
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
        "imui-popup-context-menu-item-clicked-idle-frames",
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
        "imui-popup-context-menu-item-clicked-idle-frames",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(open.get());
    assert!(!clicked.get());

    let item_bounds = bounds_for_test_id(&ui, "imui-popup-ctx-item-toggle-idle-frames");
    let click_point = Point::new(
        Px(item_bounds.origin.x.0 + item_bounds.size.width.0 * 0.5),
        Px(item_bounds.origin.y.0 + item_bounds.size.height.0 * 0.5),
    );

    for _ in 0..12 {
        app.advance_frame();
    }

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
        "imui-popup-context-menu-item-clicked-idle-frames",
        |cx| render(cx, &open_out, &clicked_out),
    );
    assert!(
        clicked.get(),
        "expected popup item click to remain observable after idle frames without render"
    );
}

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
