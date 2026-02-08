use std::sync::Arc;

use fret_ui_headless::table::{
    BuiltInFilterFn, BuiltInSortingFn, ColumnDef, PaginationState, RowKey, SortSpec, Table,
    TableOptions, TableState, TanStackSortedFlatRowOrderCache, TanStackUngroupedRowModelOrderCache,
    TanStackValue,
};

#[derive(Debug)]
struct Row {
    id: u32,
    name: &'static str,
    children: Vec<Row>,
}

fn col_id() -> ColumnDef<Row> {
    ColumnDef::<Row>::new("id")
        .sort_value_by(|row: &Row| TanStackValue::Number(row.id as f64))
        .sorting_fn_builtin(BuiltInSortingFn::Basic)
}

fn col_name() -> ColumnDef<Row> {
    ColumnDef::<Row>::new("name")
        .sort_value_by(|row: &Row| TanStackValue::String(Arc::<str>::from(row.name)))
        .sorting_fn_auto()
        .filtering_fn_builtin(BuiltInFilterFn::IncludesString)
}

#[test]
fn tanstack_v8_memo_rebuild_each_frame_guardrail_sorted_flat_row_order_ignores_pagination_state() {
    let data = vec![
        Row {
            id: 2,
            name: "b",
            children: Vec::new(),
        },
        Row {
            id: 1,
            name: "a",
            children: Vec::new(),
        },
        Row {
            id: 3,
            name: "c",
            children: Vec::new(),
        },
    ];

    let mut cache = TanStackSortedFlatRowOrderCache::default();

    let mut state = TableState::default();
    state.sorting = vec![SortSpec {
        column: "name".into(),
        desc: false,
    }];
    state.pagination = PaginationState {
        page_index: 0,
        page_size: 1,
    };

    let table = Table::builder(&data)
        .columns(vec![col_name()])
        .state(state.clone())
        .build();
    let (_order, recomputed) = table.tanstack_sorted_flat_row_order_with_cache(1, &mut cache);
    assert!(recomputed);
    assert_eq!(cache.recompute_count(), 1);
    assert_eq!(cache.filtered_recompute_count(), 1);

    // Changing pagination state should not invalidate the filter/sort root ordering cache.
    state.pagination = PaginationState {
        page_index: 1,
        page_size: 1,
    };
    let table = Table::builder(&data)
        .columns(vec![col_name()])
        .state(state)
        .build();
    let (_order, recomputed) = table.tanstack_sorted_flat_row_order_with_cache(1, &mut cache);
    assert!(!recomputed);
    assert_eq!(cache.recompute_count(), 1);
    assert_eq!(cache.filtered_recompute_count(), 1);
}

#[test]
fn tanstack_v8_memo_rebuild_each_frame_guardrail_ungrouped_row_model_order_recomputes_on_expand_and_page()
 {
    let data = vec![
        Row {
            id: 1,
            name: "root",
            children: vec![
                Row {
                    id: 10,
                    name: "c0",
                    children: Vec::new(),
                },
                Row {
                    id: 11,
                    name: "c1",
                    children: Vec::new(),
                },
            ],
        },
        Row {
            id: 2,
            name: "other",
            children: Vec::new(),
        },
    ];

    let options = TableOptions {
        paginate_expanded_rows: false,
        ..Default::default()
    };

    let mut cache = TanStackUngroupedRowModelOrderCache::default();

    let mut state = TableState::default();
    state.expanding = [RowKey(1)].into_iter().collect();
    state.pagination = PaginationState {
        page_index: 0,
        page_size: 1,
    };

    // Stable rebuilds do not recompute.
    for frame in 0..6 {
        let table = Table::builder(&data)
            .columns(vec![col_id(), col_name()])
            .get_row_key(|n, _i, _p| RowKey(n.id as u64))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .options(options)
            .state(state.clone())
            .build();
        let (_snapshot, recomputed) = table
            .tanstack_ungrouped_row_model_order_with_cache(1, &mut cache)
            .expect("ungrouped cache is available");
        assert_eq!(recomputed, frame == 0);
    }
    assert_eq!(cache.recompute_count(), 1);

    // Changing pagination should invalidate the final order.
    state.pagination.page_index = 1;
    let table = Table::builder(&data)
        .columns(vec![col_id(), col_name()])
        .get_row_key(|n, _i, _p| RowKey(n.id as u64))
        .get_sub_rows(|n, _i| Some(n.children.as_slice()))
        .options(options)
        .state(state.clone())
        .build();
    let (_snapshot, recomputed) = table
        .tanstack_ungrouped_row_model_order_with_cache(1, &mut cache)
        .expect("ungrouped cache is available");
    assert!(recomputed);
    assert_eq!(cache.recompute_count(), 2);

    // Changing expanding should also invalidate.
    state.expanding = [RowKey(2)].into_iter().collect();
    let table = Table::builder(&data)
        .columns(vec![col_id(), col_name()])
        .get_row_key(|n, _i, _p| RowKey(n.id as u64))
        .get_sub_rows(|n, _i| Some(n.children.as_slice()))
        .options(options)
        .state(state)
        .build();
    let (_snapshot, recomputed) = table
        .tanstack_ungrouped_row_model_order_with_cache(1, &mut cache)
        .expect("ungrouped cache is available");
    assert!(recomputed);
    assert_eq!(cache.recompute_count(), 3);
}
