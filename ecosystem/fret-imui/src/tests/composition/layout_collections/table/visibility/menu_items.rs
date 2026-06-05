use super::*;

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
