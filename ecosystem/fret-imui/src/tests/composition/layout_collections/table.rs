use super::*;

mod header;
mod visibility;
#[test]
fn table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(520.0), Px(240.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let columns = [
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)),
        TableColumn::px("Owner", Px(88.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-layout",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-layout",
                    &columns,
                    TableOptions {
                        striped: true,
                        test_id: Some(Arc::from("imui-table-layout")),
                        ..Default::default()
                    },
                    |table| {
                        table.row("alpha", |row| {
                            row.cell_text(
                                "Extremely long inspector label that should remain clipped inside the first fill column",
                            );
                            row.cell_text("Ready");
                            row.cell_text("Alice");
                        });
                        table.row("beta", |row| {
                            row.cell_text("Short");
                            row.cell_text("Busy");
                            row.cell_text("Bob");
                        });
                    },
                );
            })
        },
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let header_status = bounds_for_test_id(&ui, "imui-table-layout.header.cell.status");
    let row0_status = bounds_for_test_id(&ui, "imui-table-layout.row.0.cell.status");
    let row1_status = bounds_for_test_id(&ui, "imui-table-layout.row.1.cell.status");
    let header_owner = bounds_for_test_id(&ui, "imui-table-layout.header.cell.owner");
    let row0_owner = bounds_for_test_id(&ui, "imui-table-layout.row.0.cell.owner");
    let row1_owner = bounds_for_test_id(&ui, "imui-table-layout.row.1.cell.owner");

    let assert_close = |label: &str, a: f32, b: f32| {
        assert!((a - b).abs() <= 0.5, "{label} drifted: left={a}, right={b}");
    };

    assert_close(
        "status x header vs row0",
        header_status.origin.x.0,
        row0_status.origin.x.0,
    );
    assert_close(
        "status x header vs row1",
        header_status.origin.x.0,
        row1_status.origin.x.0,
    );
    assert_close(
        "status width header vs row0",
        header_status.size.width.0,
        row0_status.size.width.0,
    );
    assert_close(
        "status width header vs row1",
        header_status.size.width.0,
        row1_status.size.width.0,
    );

    assert_close(
        "owner x header vs row0",
        header_owner.origin.x.0,
        row0_owner.origin.x.0,
    );
    assert_close(
        "owner x header vs row1",
        header_owner.origin.x.0,
        row1_owner.origin.x.0,
    );
    assert_close(
        "owner width header vs row0",
        header_owner.size.width.0,
        row0_owner.size.width.0,
    );
    assert_close(
        "owner width header vs row1",
        header_owner.size.width.0,
        row1_owner.size.width.0,
    );
}

#[test]
fn table_helper_pins_left_and_right_columns_while_center_columns_scroll() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(320.0), Px(180.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();
    let scroll = fret_ui::scroll::ScrollHandle::default();

    let columns = [
        TableColumn::px("ID###id", Px(48.0)).pinned_left(),
        TableColumn::px("Name###name", Px(180.0)),
        TableColumn::px("Kind###kind", Px(160.0)),
        TableColumn::px("Score###score", Px(64.0)).pinned_right(),
    ];

    let build = |cx: &mut ElementContext<'_, TestHost>| {
        let scroll = scroll.clone();
        crate::imui_raw(cx, |ui| {
            ui.table_with_options(
                "imui-table-pinned-columns",
                &columns,
                TableOptions {
                    horizontal_scroll: Some(scroll),
                    test_id: Some(Arc::from("imui-table-pinned-columns")),
                    ..Default::default()
                },
                |table| {
                    table.row("alpha", |row| {
                        row.cell_text("01");
                        row.cell_text("Alpha asset");
                        row.cell_text("Texture");
                        row.cell_text("98");
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
        "imui-table-pinned-columns",
        build,
    );

    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let before_id = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.id");
    let before_name = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.name");
    let before_score = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.score");

    assert_eq!(columns[0].pin(), TableColumnPin::Left);
    assert_eq!(columns[3].pin(), TableColumnPin::Right);
    assert!(
        scroll.max_offset().x.0 > 0.0,
        "expected center columns to create a horizontal scroll range"
    );

    scroll.set_offset(Point::new(Px(96.0), Px(0.0)));
    let root = render_root(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-pinned-columns",
        build,
    );
    ui.set_root(root);
    ui.request_semantics_snapshot();
    ui.layout_all(&mut app, &mut services, bounds, 1.0);

    let after_id = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.id");
    let after_name = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.name");
    let after_score = bounds_for_test_id(&ui, "imui-table-pinned-columns.row.0.cell.score");

    assert!(
        (after_id.origin.x.0 - before_id.origin.x.0).abs() <= 0.5,
        "left pinned column should not move with center scroll: before={before_id:?} after={after_id:?}"
    );
    assert!(
        (after_score.origin.x.0 - before_score.origin.x.0).abs() <= 0.5,
        "right pinned column should not move with center scroll: before={before_score:?} after={after_score:?}"
    );
    assert!(
        after_name.origin.x.0 < before_name.origin.x.0 - 8.0,
        "center column should move left with horizontal scroll: before={before_name:?} after={after_name:?}"
    );
}

#[test]
fn table_helper_applies_explicit_row_and_cell_background_overrides() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(360.0), Px(160.0)),
    );

    let mut ui = UiTree::new();
    ui.set_window(window);

    let mut app = TestHost::new();
    app.set_global(PlatformCapabilities::default());
    let mut services = FakeTextService::default();

    let row_bg = fret_core::Color::from_srgb_hex_rgb(0x201010);
    let cell_bg = fret_core::Color::from_srgb_hex_rgb(0x102020);
    let columns = [
        TableColumn::fill("Name"),
        TableColumn::px("Status", Px(96.0)),
    ];

    let _root = run_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        "imui-table-background-overrides",
        |cx| {
            crate::imui_raw(cx, |ui| {
                ui.table_with_options(
                    "imui-table-background-overrides",
                    &columns,
                    TableOptions {
                        striped: true,
                        test_id: Some(Arc::from("imui-table-background-overrides")),
                        ..Default::default()
                    },
                    |table| {
                        table.row_with_options(
                            "alpha",
                            fret_ui_kit::imui::TableRowOptions {
                                test_id: Some(Arc::from("imui-table-background-overrides.row")),
                                background: Some(row_bg),
                            },
                            |row| {
                                row.cell_text("Alpha");
                                row.cell_text_with_options(
                                    "Ready",
                                    fret_ui_kit::imui::TableCellOptions {
                                        test_id: Some(Arc::from(
                                            "imui-table-background-overrides.cell",
                                        )),
                                        background: Some(cell_bg),
                                    },
                                );
                            },
                        );
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
        "imui-table-background-overrides.row"
    ));
    assert!(has_test_id(
        &mut ui,
        &mut app,
        &mut services,
        bounds,
        "imui-table-background-overrides.cell"
    ));

    services.prepared.clear();
    let mut scene = fret_core::Scene::default();
    ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);
    let ops = scene.ops();

    let row_bg_index = first_solid_quad_index(ops, row_bg).expect("row background quad");
    let cell_bg_index = first_solid_quad_index(ops, cell_bg).expect("cell background quad");
    assert!(
        row_bg_index < cell_bg_index,
        "expected cell background to paint after row background, got scene ops: {ops:?}"
    );
}

fn first_solid_quad_index(ops: &[fret_core::SceneOp], color: fret_core::Color) -> Option<usize> {
    ops.iter().position(|op| {
        matches!(
            op,
            fret_core::SceneOp::Quad {
                background:
                    fret_core::scene::PaintBindingV1 {
                        paint: fret_core::scene::Paint::Solid(actual),
                        ..
                    },
                ..
            } if *actual == color
        )
    })
}
