use super::*;

#[test]
fn table_sortable_header_reports_app_owned_trigger_without_sorting_rows() {
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

    let name_clicked = Rc::new(Cell::new(false));
    let status_clicked = Rc::new(Cell::new(false));

    let render = |cx: &mut ElementContext<'_, TestHost>,
                  name_clicked_out: &Rc<Cell<bool>>,
                  status_clicked_out: &Rc<Cell<bool>>| {
        crate::imui_raw(cx, |ui| {
            let columns = [
                TableColumn::fill("Name###asset-name")
                    .sortable()
                    .sorted(TableSortDirection::Ascending),
                TableColumn::px("Status###asset-status", Px(120.0)).sortable(),
            ];
            let response = ui.table_with_options(
                "sortable-table",
                &columns,
                TableOptions {
                    test_id: Some(Arc::from("imui-table-sort")),
                    ..Default::default()
                },
                |table| {
                    table.row("asset-a", |row| {
                        row.cell_text("Asset A");
                        row.cell_text("Ready");
                    });
                    table.row("asset-b", |row| {
                        row.cell_text("Asset B");
                        row.cell_text("Busy");
                    });
                },
            );

            let name = response
                .header("asset-name")
                .expect("asset-name sortable header response");
            assert_eq!(name.column_index(), 0);
            assert!(name.sortable());
            assert_eq!(name.column_id(), Some("asset-name"));
            assert_eq!(name.sort_direction(), Some(TableSortDirection::Ascending));
            name_clicked_out.set(name.clicked());

            let status = response
                .header("asset-status")
                .expect("asset-status sortable header response");
            assert_eq!(status.column_index(), 1);
            assert!(status.sortable());
            assert_eq!(status.sort_direction(), None);
            status_clicked_out.set(status.clicked());
        })
    };

    let name_clicked_out = name_clicked.clone();
    let status_clicked_out = status_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-sortable-header",
        |cx| render(cx, &name_clicked_out, &status_clicked_out),
    );
    assert!(!name_clicked.get());
    assert!(!status_clicked.get());

    assert!(services.prepared.iter().any(|text| text == "Name"));
    assert!(services.prepared.iter().any(|text| text == "^"));
    assert!(services.prepared.iter().any(|text| text == "Status"));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-sort.header.cell.asset-name"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-sort.header.cell.asset-status"
    ));
    assert!(
        !services.prepared.iter().any(|text| text.contains("###")
            || text.contains("asset-name")
            || text.contains("asset-status")),
        "sortable table header suffixes should not be painted: {:?}",
        services.prepared
    );

    let click_point = point_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-sort.header.cell.asset-name",
    );
    click_at(&mut ui, &mut app, &mut services, click_point);

    app.advance_frame();
    let name_clicked_out = name_clicked.clone();
    let status_clicked_out = status_clicked.clone();
    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-sortable-header",
        |cx| render(cx, &name_clicked_out, &status_clicked_out),
    );

    assert!(name_clicked.get());
    assert!(!status_clicked.get());
}
