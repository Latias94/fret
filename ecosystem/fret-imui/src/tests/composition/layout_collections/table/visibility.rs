use super::*;

#[test]
fn table_helper_skips_hidden_columns_in_header_and_body() {
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
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)).hidden(),
        TableColumn::px("Owner", Px(88.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-hidden-column",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-hidden-column",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-hidden-column")),
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
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.name"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.owner"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.header.cell.status"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-hidden-column.row.0.cell.status"
    ));
}

#[test]
fn table_helper_applies_runtime_column_visibility_state() {
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
        TableColumn::px("Owner###owner", Px(88.0)).hidden(),
    ];
    let visibility = ImUiTableColumnVisibilityState::new([
        (Arc::from("status"), false),
        (Arc::from("owner"), true),
    ]);
    let columns = visibility.apply_to_columns(&columns);

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-runtime-column-visibility",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-runtime-column-visibility",
                    &columns,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-runtime-column-visibility")),
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
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.name"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.status"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.header.cell.owner"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.name"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.status"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-runtime-column-visibility.row.0.cell.owner"
    ));
}

#[test]
fn table_column_visibility_menu_item_updates_visibility_state() {
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

    let model = app
        .models_mut()
        .insert(ImUiTableColumnVisibilityState::default());
    let column = TableColumn::px("Status###status", Px(96.0));

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let _ = table_column_visibility_menu_item(
                    ui,
                    &column,
                    &model,
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu.status")),
                        ..Default::default()
                    },
                );
            })
        },
    );

    let item = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu.status",
    );
    click_at(&mut ui, &mut app, &mut services, item);

    app.advance_frame();
    let changed = Rc::new(Cell::new(false));
    let visible = Rc::new(Cell::new(true));
    let changed_out = changed.clone();
    let visible_out = visible.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_item(
                    ui,
                    &column,
                    &model,
                    MenuItemOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu.status")),
                        ..Default::default()
                    },
                )
                .expect("column has stable id");
                changed_out.set(response.changed());
                let value = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .map(|state| state.is_visible("status", column.visible()))
                    .unwrap_or(column.visible());
                visible_out.set(value);
            })
        },
    );

    assert!(changed.get());
    assert!(!visible.get());
}

#[test]
fn table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns() {
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
        TableColumn::unlabeled(TableColumnWidth::px(Px(64.0))).with_id("actions"),
        TableColumn::px("###internal", Px(48.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_items(
                    ui,
                    &columns,
                    &model,
                    TableColumnVisibilityMenuOptions {
                        test_id_prefix: Some(Arc::from(
                            "imui-table-column-visibility-menu-items.item.",
                        )),
                        ..Default::default()
                    },
                );
                assert_eq!(response.len(), 2);
                assert!(response.item("name").is_some());
                assert!(response.item("status").is_some());
                assert!(response.item("actions").is_none());
                assert!(response.item("internal").is_none());
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.name",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.status",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.actions",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.internal",
    ));

    let status = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items.item.status",
    );
    click_at(&mut ui, &mut app, &mut services, status);

    app.advance_frame();
    let changed = Rc::new(Cell::new(false));
    let visible = Rc::new(Cell::new(true));
    let changed_out = changed.clone();
    let visible_out = visible.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let response = table_column_visibility_menu_items(
                    ui,
                    &columns,
                    &model,
                    TableColumnVisibilityMenuOptions {
                        test_id_prefix: Some(Arc::from(
                            "imui-table-column-visibility-menu-items.item.",
                        )),
                        ..Default::default()
                    },
                );
                changed_out.set(response.changed());
                visible_out.set(
                    response
                        .item("status")
                        .expect("status item response")
                        .visible(),
                );
            })
        },
    );

    assert!(changed.get());
    assert!(!visible.get());

    let applied_visible = app
        .models()
        .get_cloned(&model)
        .expect("visibility model")
        .apply_to_columns(&columns);
    assert!(applied_visible[0].visible());
    assert!(!applied_visible[1].visible());
    assert!(applied_visible[2].visible());
    assert!(applied_visible[3].visible());

    app.advance_frame();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-column-visibility-menu-items",
        |cx| {
            crate::imui_raw(cx, |ui| {
                let applied = ui
                    .cx_mut()
                    .app
                    .models()
                    .get_cloned(&model)
                    .expect("visibility model")
                    .apply_to_columns(&columns);
                ui.table_with_options(
                    "imui-table-column-visibility-menu-items-applied",
                    &applied,
                    TableOptions {
                        test_id: Some(Arc::from("imui-table-column-visibility-menu-items-applied")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text("Alpha");
                            row.cell_text("Ready");
                            row.cell_text("Open");
                            row.cell_text("Internal");
                        });
                    },
                );
            })
        },
    );

    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.header.cell.name",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.header.cell.status",
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.row.0.cell.name",
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-column-visibility-menu-items-applied.row.0.cell.status",
    ));
}

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
