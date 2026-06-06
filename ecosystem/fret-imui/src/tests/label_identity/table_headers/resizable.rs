use super::*;

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
            let resize = name.resize();
            assert_eq!(resize.column_index(), 0);
            assert_eq!(resize.column_id(), Some("asset-name"));
            assert_eq!(resize.min_width(), Some(Px(80.0)));
            assert_eq!(resize.max_width(), Some(Px(240.0)));
            resize_enabled.set(resize.enabled());
            resize_dragging.set(resize.dragging());
            resize_started.set(resize.drag_started());
            resize_stopped.set(resize.drag_stopped());
            resize_total_x.set(resize.drag_total_x());
            resize_width.set(resize.width_from_start(Px(140.0)).0);
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
