use super::*;

#[test]
fn table_plain_header_left_click_does_not_activate_or_click() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let clicked = Rc::new(Cell::new(true));
    let activated = Rc::new(Cell::new(true));
    let deactivated = Rc::new(Cell::new(true));
    let render = {
        let clicked = clicked.clone();
        let activated = activated.clone();
        let deactivated = deactivated.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-plain-header-left-click",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-left-click")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                clicked.set(header.clicked());
                activated.set(header.activated());
                deactivated.set(header.deactivated());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-left-click",
        &render,
    );
    assert!(!clicked.get());
    assert!(!activated.get());
    assert!(!deactivated.get());

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-left-click.header.cell.name",
    );
    click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-left-click",
        &render,
    );
    assert!(!clicked.get());
    assert!(!activated.get());
    assert!(!deactivated.get());
}

#[test]
fn table_plain_header_reports_context_menu_request_from_keyboard_without_clicking() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let clicked = Rc::new(Cell::new(false));
    let header_id = Rc::new(Cell::new(None));
    let header_id_out = header_id.clone();

    let render = {
        let requested = requested.clone();
        let clicked = clicked.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-plain-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                header_id_out.set(header.id());
                requested.set(header.context_menu_requested());
                clicked.set(header.clicked());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(!requested.get());
    assert!(!clicked.get());

    let header_id = header_id.get().expect("plain header response id");
    ui.request_focus_element(&mut app, header_id);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.focus().is_some(),
        "expected plain header trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(requested.get());
    assert!(!clicked.get());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(!requested.get());
    assert!(!clicked.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-keyboard-context-menu",
        &render,
    );
    assert!(requested.get());
    assert!(!clicked.get());
}

#[test]
fn table_sortable_header_reports_context_menu_request_on_right_click() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let anchor_matches_click = Rc::new(Cell::new(false));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                assert!(
                    !response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested()
                );
            })
        },
    );

    let at = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-context-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, at);

    app.advance_frame();
    let requested_out = requested.clone();
    let anchor_matches_click_out = anchor_matches_click.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                let header = response.header("name").expect("name header").response();
                requested_out.set(header.context_menu_requested());
                anchor_matches_click_out.set(header.context_menu_anchor() == Some(at));
            })
        },
    );

    assert!(requested.get());
    assert!(anchor_matches_click.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(!requested.get());
}

#[test]
fn table_sortable_header_reports_context_menu_request_from_keyboard() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
    ];
    let requested = Rc::new(Cell::new(false));
    let header_id = Rc::new(Cell::new(None));
    let header_id_out = header_id.clone();

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                header_id_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .id(),
                );
                assert!(
                    !response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested()
                );
            })
        },
    );

    let header_id = header_id.get().expect("name header response id");
    ui.request_focus_element(&mut app, header_id);
    ui.layout_all(&mut app, &mut services, bounds, 1.0);
    assert!(
        ui.focus().is_some(),
        "expected sortable header trigger to take focus"
    );

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::ContextMenu,
        Modifiers::default(),
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(requested.get());

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(!requested.get());

    key_down(
        &mut ui,
        &mut app,
        &mut services,
        KeyCode::F10,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
    );

    app.advance_frame();
    let requested_out = requested.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-keyboard-context-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = ui.table_with_options(
                    "imui-table-header-keyboard-context-menu",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-keyboard-context-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                        });
                    },
                );
                requested_out.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .context_menu_requested(),
                );
            })
        },
    );

    assert!(requested.get());
}
