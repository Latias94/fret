use super::*;

#[test]
fn table_column_visibility_header_context_menu_opens_and_updates_state() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let columns = [
        TableColumn::fill("Name###name").sortable(),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];

    let opened = Rc::new(Cell::new(false));
    let render = {
        let model = model.clone();
        let opened = opened.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                let response = ui.table_with_options(
                    "imui-table-header-visibility-menu",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-header-visibility-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
                let menu = table_column_visibility_header_context_menu(
                    ui,
                    "imui-table-header-visibility-menu.columns",
                    &response,
                    &columns,
                    &model,
                    TableColumnVisibilityHeaderContextMenuOptions {
                        menu: TableColumnVisibilityMenuOptions {
                            test_id_prefix: Some(Arc::from(
                                "imui-table-header-visibility-menu.menu.item.",
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );
                opened.set(menu.open());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(!opened.get());
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    ));

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(opened.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.name",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    ));

    let status_item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.menu.item.status",
    );
    click_at(&mut ui, &mut app, &mut services, status_item);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(
        !app.models()
            .get_cloned(&model)
            .expect("visibility model")
            .is_visible("status", true)
    );

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-header-visibility-menu",
        &render,
    );
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.header.cell.status",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-header-visibility-menu.row.0.cell.status",
    ));
}

#[test]
fn table_column_visibility_header_context_menu_opens_from_plain_header() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(420.0), Px(220.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let columns = [
        TableColumn::fill("Name###name"),
        TableColumn::px("Status###status", Px(96.0)),
        TableColumn::px("Owner###owner", Px(88.0)),
    ];

    let opened = Rc::new(Cell::new(false));
    let plain_header_clicked = Rc::new(Cell::new(true));
    let render = {
        let model = model.clone();
        let opened = opened.clone();
        let plain_header_clicked = plain_header_clicked.clone();
        move |cx: &mut ElementContext<'_, TestHost>| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                let response = ui.table_with_options(
                    "imui-table-plain-header-visibility-menu",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-plain-header-visibility-menu")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                    },
                );
                plain_header_clicked.set(
                    response
                        .header("name")
                        .expect("name header")
                        .response()
                        .clicked(),
                );
                let menu = table_column_visibility_header_context_menu(
                    ui,
                    "imui-table-plain-header-visibility-menu.columns",
                    &response,
                    &columns,
                    &model,
                    TableColumnVisibilityHeaderContextMenuOptions {
                        menu: TableColumnVisibilityMenuOptions {
                            test_id_prefix: Some(Arc::from(
                                "imui-table-plain-header-visibility-menu.menu.item.",
                            )),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                );
                opened.set(menu.open());
            })
        }
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-visibility-menu",
        &render,
    );
    assert!(!opened.get());
    assert!(!plain_header_clicked.get());

    let header = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-visibility-menu.header.cell.name",
    );
    right_click_at(&mut ui, &mut app, &mut services, header);

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-plain-header-visibility-menu",
        &render,
    );
    assert!(opened.get());
    assert!(!plain_header_clicked.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-plain-header-visibility-menu.menu.item.status",
    ));
}
