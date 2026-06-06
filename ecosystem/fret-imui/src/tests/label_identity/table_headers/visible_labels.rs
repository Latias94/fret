use super::*;

#[test]
fn label_identity_table_headers_hide_suffixes_from_visible_labels() {
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

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let columns = [
                TableColumn::fill("Name##asset-name-column"),
                TableColumn::px("Status###status-column", Px(120.0)),
                TableColumn::unlabeled(fret_ui_kit::imui::TableColumnWidth::px(Px(64.0)))
                    .with_id("row-actions"),
            ];
            ui.table_with_options(
                "identity-table",
                &columns,
                TableOptions {
                    test_id: Some(Arc::from("imui-label-identity.table")),
                    ..Default::default()
                },
                |table| {
                    table.row("asset-a", |row| {
                        row.cell_text("Asset A");
                        row.cell_text("Ready");
                        row.cell_text("Open");
                    });
                },
            );
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-label-identity-table-headers",
        |cx| render(cx),
    );

    assert!(services.prepared.iter().any(|text| text == "Name"));
    assert!(services.prepared.iter().any(|text| text == "Status"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.header.cell.name-asset-name-column"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.header.cell.status-column"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.header.cell.row-actions"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.row.0.cell.name-asset-name-column"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.row.0.cell.status-column"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.row.0.cell.row-actions"
    ));
    assert!(!has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-label-identity.table.header.cell.0"
    ));
    assert!(
        !services.prepared.iter().any(|text| text.contains("##")
            || text.contains("###")
            || text.contains("asset-name-column")
            || text.contains("status-column")),
        "table header label suffixes should not be painted: {:?}",
        services.prepared
    );
}
