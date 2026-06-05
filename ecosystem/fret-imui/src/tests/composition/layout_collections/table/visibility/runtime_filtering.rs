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
