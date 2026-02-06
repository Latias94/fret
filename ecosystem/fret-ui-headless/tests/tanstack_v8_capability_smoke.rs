use fret_ui_headless::table::{
    ColumnDef, ColumnPinPosition, ColumnSizingRegion, RowKey, RowPinPosition, Table, TableState,
};

#[derive(Debug, Clone)]
struct DemoRow {
    id: u64,
    a: u64,
    b: u64,
}

#[test]
fn tanstack_v8_capability_smoke_table_row_column_surfaces_exist() {
    let data = vec![
        DemoRow {
            id: 1,
            a: 10,
            b: 100,
        },
        DemoRow {
            id: 2,
            a: 20,
            b: 200,
        },
        DemoRow {
            id: 3,
            a: 30,
            b: 300,
        },
    ];

    let columns = vec![
        ColumnDef::<DemoRow>::new("a").value_u64_by(|r| r.a),
        ColumnDef::<DemoRow>::new("b").value_u64_by(|r| r.b),
    ];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .state(TableState::default())
        .build();

    // Core row model surface.
    let _core = table.core_row_model();
    let _final = table.row_model();

    // Header/cell/id surfaces used by UI recipes.
    let _ = table.core_model_snapshot();
    let _ = table.header_groups();
    let _ = table.left_header_groups();
    let _ = table.center_header_groups();
    let _ = table.right_header_groups();

    let _ = table.header_size("a");
    let _ = table.header_start("a");

    // Column sizing offsets (pinned-aware).
    let _ = table.column_size("a");
    let _ = table.column_start("a", ColumnSizingRegion::Center);
    let _ = table.column_after("a", ColumnSizingRegion::Center);

    // Column pinning.
    let _ = table.column_pin_position("a");
    let next_col_pinning = table
        .toggled_column_pinning("a", Some(ColumnPinPosition::Left))
        .expect("column exists");
    assert!(!next_col_pinning.left.is_empty());

    // Row pinning (table-level updater).
    let updater = table.row_pinning_updater(RowKey(2), Some(RowPinPosition::Top), false, false);
    let next_row_pinning = updater.apply(&table.state().row_pinning);
    assert_eq!(next_row_pinning.top, vec![RowKey(2)]);

    // Row/cell split snapshots.
    let _ = table.row_cells(RowKey(1)).expect("row exists");
    let _top = table.top_row_keys();
    let _center = table.center_row_keys();
    let _bottom = table.bottom_row_keys();
}
