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
