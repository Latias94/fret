use fret_ui_headless::table::{
    ColumnDef, ColumnPinPosition, ColumnSizingRegion, ExpandingState, RowId, RowKey,
    RowPinPosition, Table, TableState, TanStackTableState, TanStackValue,
};
use std::sync::Arc;

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
        ColumnDef::<DemoRow>::new("a")
            .sort_value_by(|r| TanStackValue::Number(r.a as f64))
            .sorting_fn_auto()
            .filtering_fn_auto(),
        ColumnDef::<DemoRow>::new("b")
            .sort_value_by(|r| TanStackValue::Number(r.b as f64))
            .sorting_fn_auto()
            .filtering_fn_auto(),
    ];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
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

    // Leaf column pin splits (TanStack-style).
    let _ = table.pinned_leaf_columns();
    let _ = table.left_leaf_columns();
    let _ = table.center_leaf_columns();
    let _ = table.right_leaf_columns();

    // Row pinning (table-level updater).
    let updater = table.row_pinning_updater(RowKey(2), Some(RowPinPosition::Top), false, false);
    let next_row_pinning = updater.apply(&table.state().row_pinning);
    assert_eq!(next_row_pinning.top, vec![RowKey(2)]);

    // Sorting handler transition (TanStack-style "getToggleSortingHandler" behavior).
    let sorting = table
        .toggled_column_sorting_handler_tanstack("a", false, false)
        .expect("column exists");
    assert!(!sorting.is_empty());

    // Column filter state transition (TanStack-style `column.setFilterValue` behavior).
    assert_eq!(table.column_can_filter("a"), Some(true));
    assert_eq!(table.column_is_filtered("a"), Some(false));
    assert_eq!(table.column_filter_index("a"), Some(-1));

    let updater = table
        .column_filters_updater_set_value("a", serde_json::json!(10))
        .expect("column exists");
    let next_filters = updater.apply(&table.state().column_filters);
    assert!(!next_filters.is_empty());

    let mut next_state = TableState::default();
    next_state.column_filters = next_filters;
    let table_with_filter = Table::builder(&data)
        .columns(table.columns().to_vec())
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
        .state(next_state)
        .build();
    assert!(table_with_filter.column_filter_value("a").is_some());
    assert_eq!(table_with_filter.column_filter_index("a"), Some(0));
    assert_eq!(table_with_filter.column_is_filtered("a"), Some(true));

    // Global filtering helper surfaces.
    assert_eq!(table.column_can_global_filter("a"), Some(true));
    let next_global_filter = table
        .global_filter_updater_set_value(Some(serde_json::json!("ap")))
        .apply(&table.state().global_filter);
    assert!(next_global_filter.is_some());

    // Row/cell split snapshots.
    let _ = table.row_cells(RowKey(1)).expect("row exists");
    let _top = table.top_row_keys();
    let _center = table.center_row_keys();
    let _bottom = table.bottom_row_keys();
    let _top_ids = table.top_row_ids();
    let _center_ids = table.center_row_ids();
    let _bottom_ids = table.bottom_row_ids();

    // Cell context snapshot (TanStack `cell.getContext()` equivalent).
    let ctx = table.cell_context(RowKey(1), "a").expect("cell context");
    assert_eq!(ctx.row_id.as_str(), "1");
    assert_eq!(ctx.column_id.as_ref(), "a");
    assert_eq!(ctx.id.as_ref(), "1_a");

    // TanStack-style value accessors.
    assert_eq!(
        table.cell_value(RowKey(1), "a"),
        Some(TanStackValue::Number(10.0))
    );
    assert_eq!(
        table.row_unique_values(RowKey(1), "a"),
        Some(vec![TanStackValue::Number(10.0)])
    );
}

#[test]
fn tanstack_v8_capability_smoke_custom_row_id_affects_lookup_and_cell_ids() {
    let data = vec![DemoRow {
        id: 1,
        a: 10,
        b: 100,
    }];
    let columns = vec![
        ColumnDef::<DemoRow>::new("a").value_u64_by(|r| r.a),
        ColumnDef::<DemoRow>::new("b").value_u64_by(|r| r.b),
    ];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)))
        .state(TableState::default())
        .build();

    assert_eq!(table.row_key_for_id("row:1", false), Some(RowKey(1)));

    let cells = table.row_cells(RowKey(1)).expect("row exists");
    assert!(
        cells
            .all
            .iter()
            .any(|c| c.id.as_ref() == "row:1_a" && c.column_id.as_ref() == "a")
    );
}

#[test]
fn tanstack_v8_capability_smoke_tanstack_state_can_resolve_string_row_ids() {
    let data = vec![DemoRow {
        id: 1,
        a: 10,
        b: 100,
    }];
    let columns = vec![
        ColumnDef::<DemoRow>::new("a").value_u64_by(|r| r.a),
        ColumnDef::<DemoRow>::new("b").value_u64_by(|r| r.b),
    ];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)))
        .state(TableState::default())
        .build();

    let tanstack = TanStackTableState::from_json(&serde_json::json!({
        "rowPinning": { "top": ["row:1"], "bottom": [] },
        "rowSelection": { "row:1": true },
        "expanded": { "row:1": true }
    }))
    .expect("tanstack state json");

    let state = tanstack
        .to_table_state_with_row_model(table.core_row_model())
        .expect("resolved state");

    assert_eq!(state.row_pinning.top, vec![RowKey(1)]);
    assert!(state.row_selection.contains(&RowKey(1)));
    assert!(matches!(state.expanding, ExpandingState::Keys(keys) if keys.contains(&RowKey(1))));

    let updater = table
        .row_pinning_updater_by_id("row:1", true, Some(RowPinPosition::Top), false, false)
        .expect("row exists");
    let next = updater.apply(&TableState::default().row_pinning);
    assert_eq!(next.top, vec![RowKey(1)]);
}

#[test]
fn tanstack_v8_capability_smoke_row_id_updaters_cover_selection_and_expanding() {
    let data = vec![DemoRow {
        id: 1,
        a: 10,
        b: 100,
    }];
    let columns = vec![
        ColumnDef::<DemoRow>::new("a").value_u64_by(|r| r.a),
        ColumnDef::<DemoRow>::new("b").value_u64_by(|r| r.b),
    ];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(format!("row:{}", row.id)))
        .state(TableState::default())
        .build();

    let selection_updater = table
        .row_selection_updater_by_id("row:1", true, Some(true), true)
        .expect("row selection updater by id");
    let next_selection = selection_updater.apply(&TableState::default().row_selection);
    assert!(next_selection.contains(&RowKey(1)));

    let next_selection_toggled = table
        .toggled_row_selected_by_id("row:1", true, Some(true), true)
        .expect("row selection toggle by id");
    assert!(next_selection_toggled.contains(&RowKey(1)));

    let expanding_updater = table
        .row_expanding_updater_by_id("row:1", true, Some(true))
        .expect("row expanding updater by id");
    let next_expanding = expanding_updater.apply(&TableState::default().expanding);
    assert!(matches!(
        next_expanding,
        ExpandingState::Keys(keys) if keys.contains(&RowKey(1))
    ));

    let next_expanding_toggled = table
        .toggled_row_expanded_by_id("row:1", true, Some(true))
        .expect("row expanding toggle by id");
    assert!(matches!(
        next_expanding_toggled,
        ExpandingState::Keys(keys) if keys.contains(&RowKey(1))
    ));
}

#[test]
fn tanstack_v8_capability_smoke_grouped_row_ids_exist_and_resolve_to_row_keys() {
    #[derive(Debug, Clone)]
    struct GroupRow {
        id: u64,
        role: u64,
    }

    let data = vec![
        GroupRow { id: 1, role: 1 },
        GroupRow { id: 2, role: 2 },
        GroupRow { id: 3, role: 1 },
    ];

    let columns = vec![
        ColumnDef::<GroupRow>::new("role").facet_key_by(|r| r.role),
        ColumnDef::<GroupRow>::new("id").value_u64_by(|r| r.id),
    ];

    let mut state = TableState::default();
    state.grouping = vec![Arc::<str>::from("role")];

    let table = Table::builder(&data)
        .columns(columns)
        .get_row_key(|row, _idx, _parent| RowKey(row.id))
        .get_row_id(|row, _idx, _parent| RowId::new(row.id.to_string()))
        .state(state)
        .build();

    let grouped = table.grouped_row_model();
    let root = grouped.root_rows().first().copied().expect("group root");
    let root_row = grouped.row(root).expect("group root row");
    assert_eq!(root_row.id.as_str(), "role:1");

    let resolved = table
        .row_key_for_id("role:1", true)
        .expect("group id resolves");
    assert_eq!(resolved, root_row.key);

    let updater = table
        .row_pinning_updater_by_id("role:1", true, Some(RowPinPosition::Top), false, false)
        .expect("group row pin updater");
    let next = updater.apply(&TableState::default().row_pinning);
    assert_eq!(next.top, vec![root_row.key]);

    let include_leaf = table
        .row_pinning_updater_by_id("role:1", true, Some(RowPinPosition::Top), true, false)
        .expect("group row pin updater (include leaf)");
    let next_include_leaf = include_leaf.apply(&TableState::default().row_pinning);
    assert!(next_include_leaf.top.contains(&root_row.key));
    assert!(next_include_leaf.top.contains(&RowKey(1)));
    assert!(next_include_leaf.top.contains(&RowKey(3)));

    let include_parent = table
        .row_pinning_updater_by_id("1", true, Some(RowPinPosition::Top), false, true)
        .expect("leaf row pin updater (include parent)");
    let next_include_parent = include_parent.apply(&TableState::default().row_pinning);
    assert!(next_include_parent.top.contains(&root_row.key));
    assert!(next_include_parent.top.contains(&RowKey(1)));

    let next_expanding = table
        .toggled_row_expanded_by_id("role:1", true, Some(true))
        .expect("group row expand updater");
    assert!(matches!(
        next_expanding,
        ExpandingState::Keys(keys) if keys.contains(&root_row.key)
    ));
}
