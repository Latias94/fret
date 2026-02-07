use fret_ui_headless::table::{
    BuiltInSortingFn, ColumnDef, SortSpec, Table, TableState, TanStackSortedFlatRowOrderCache,
    TanStackValue,
};

#[derive(Debug)]
struct Row {
    id: u32,
}

fn col_id() -> ColumnDef<Row> {
    ColumnDef::<Row>::new("id")
        .sort_value_by(|row: &Row| TanStackValue::Number(row.id as f64))
        .sorting_fn_builtin(BuiltInSortingFn::Basic)
}

#[test]
fn tanstack_v8_perf_large_dataset_rebuild_each_frame_cache_avoids_recompute() {
    // This is a perf guardrail, not a micro-benchmark:
    // - large input size
    // - repeated rebuilds of `Table`
    // - persistent memo cache must prevent per-frame recomputation.
    const N: u32 = 50_000;
    const FRAMES: usize = 40;

    let data: Vec<Row> = (0..N).rev().map(|id| Row { id }).collect();

    let state = TableState {
        sorting: vec![SortSpec {
            column: "id".into(),
            desc: false,
        }],
        ..TableState::default()
    };

    let mut cache = TanStackSortedFlatRowOrderCache::default();

    for _frame in 0..FRAMES {
        let table = Table::builder(&data)
            .columns(vec![col_id()])
            .state(state.clone())
            .build();
        let (_order, _recomputed) = table.tanstack_sorted_flat_row_order_with_cache(1, &mut cache);
    }

    assert_eq!(cache.recompute_count(), 1);
}
