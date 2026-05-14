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
            assert_eq!(name.column_index, 0);
            assert!(name.sortable);
            assert_eq!(name.column_id(), Some("asset-name"));
            assert_eq!(name.sort_direction, Some(TableSortDirection::Ascending));
            name_clicked_out.set(name.clicked());

            let status = response
                .header("asset-status")
                .expect("asset-status sortable header response");
            assert_eq!(status.column_index, 1);
            assert!(status.sortable);
            assert_eq!(status.sort_direction, None);
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

#[test]
fn table_resizable_header_reports_drag_response() {
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

    let resize_enabled = Rc::new(Cell::new(false));
    let resize_dragging = Rc::new(Cell::new(false));
    let resize_started = Rc::new(Cell::new(false));
    let resize_stopped = Rc::new(Cell::new(false));
    let resize_total_x = Rc::new(Cell::new(0.0f32));
    let resize_width = Rc::new(Cell::new(0.0f32));

    let render = |cx: &mut ElementContext<'_, TestHost>| {
        crate::imui_raw(cx, |ui| {
            let columns = [
                TableColumn::px("Name###asset-name", Px(140.0))
                    .resizable_with_limits(Some(Px(80.0)), Some(Px(240.0))),
                TableColumn::px("Status###asset-status", Px(120.0)),
            ];
            let response = ui.table_with_options(
                "resizable-table",
                &columns,
                TableOptions {
                    test_id: Some(Arc::from("imui-table-resize")),
                    ..Default::default()
                },
                |table| {
                    table.row("asset-a", |row| {
                        row.cell_text("Asset A");
                        row.cell_text("Ready");
                    });
                },
            );

            let name = response
                .header("asset-name")
                .expect("asset-name header response");
            assert_eq!(name.resize.column_index, 0);
            assert_eq!(name.resize.column_id(), Some("asset-name"));
            assert_eq!(name.resize.min_width, Some(Px(80.0)));
            assert_eq!(name.resize.max_width, Some(Px(240.0)));
            resize_enabled.set(name.resize.enabled());
            resize_dragging.set(name.resize.dragging());
            resize_started.set(name.resize.drag_started());
            resize_stopped.set(name.resize.drag_stopped());
            resize_total_x.set(name.resize.drag_total_x());
            resize_width.set(name.resize.width_from_start(Px(140.0)).0);
        })
    };

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-resizable-header",
        |cx| render(cx),
    );
    assert!(resize_enabled.get());
    assert!(!resize_dragging.get());
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-resize.header.cell.asset-name.resize"
    ));

    let handle_node = node_for_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-resize.header.cell.asset-name.resize",
    );
    let handle_bounds = ui.debug_node_bounds(handle_node).expect("handle bounds");
    let handle = Point::new(
        Px(handle_bounds.origin.x.0 + handle_bounds.size.width.0 / 2.0),
        Px(handle_bounds.origin.y.0 + handle_bounds.size.height.0 / 2.0),
    );
    let hit = ui.debug_hit_test(handle).hit;
    let hit_path = hit.map(|node| ui.debug_node_path(node)).unwrap_or_default();
    assert!(
        hit_path.contains(&handle_node),
        "resize handle point should route through the resize handle bounds={handle_bounds:?} hit={hit:?} path={hit_path:?}"
    );
    pointer_down_at(&mut ui, &mut app, &mut services, handle);

    let drag_to = Point::new(Px(handle.x.0 + 48.0), handle.y);
    let mut buttons = MouseButtons::default();
    buttons.left = true;
    pointer_move_at(&mut ui, &mut app, &mut services, drag_to, buttons);

    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-resizable-header",
        &render,
    );

    assert!(
        resize_started.get(),
        "resize did not start: dragging={} total_x={} width={}",
        resize_dragging.get(),
        resize_total_x.get(),
        resize_width.get()
    );
    assert!(
        resize_dragging.get(),
        "resize not dragging: total_x={} width={}",
        resize_total_x.get(),
        resize_width.get()
    );
    assert!(resize_total_x.get() >= 40.0);
    assert!(resize_width.get() >= 180.0);

    pointer_up_at(&mut ui, &mut app, &mut services, drag_to);
    let _root = advance_and_run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-resizable-header",
        &render,
    );

    assert!(resize_stopped.get());
    assert!(!resize_dragging.get());
}
