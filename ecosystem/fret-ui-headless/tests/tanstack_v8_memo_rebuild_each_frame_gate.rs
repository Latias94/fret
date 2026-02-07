use std::sync::Arc;

use fret_ui_headless::table::{
    ColumnDef, SortSpec, Table, TableState, TanStackSortedFlatRowOrderCache, TanStackValue,
};

#[derive(Debug)]
struct Row {
    name: &'static str,
}

fn col_name() -> ColumnDef<Row> {
    ColumnDef::<Row>::new("name")
        .sort_value_by(|row: &Row| TanStackValue::String(Arc::<str>::from(row.name)))
        .sorting_fn_auto()
}

#[test]
fn tanstack_v8_memo_rebuild_each_frame_sorted_flat_row_order_cache_is_reused() {
    let data = vec![Row { name: "b" }, Row { name: "a" }, Row { name: "c" }];

    let state = TableState {
        sorting: vec![SortSpec {
            column: "name".into(),
            desc: false,
        }],
        ..TableState::default()
    };

    let mut cache = TanStackSortedFlatRowOrderCache::default();

    let mut recompute_flags = Vec::new();
    for _frame in 0..8 {
        let table = Table::builder(&data)
            .columns(vec![col_name()])
            .state(state.clone())
            .build();
        let (_order, recomputed) = table.tanstack_sorted_flat_row_order_with_cache(1, &mut cache);
        recompute_flags.push(recomputed);
    }

    assert_eq!(recompute_flags.first().copied(), Some(true));
    assert!(
        recompute_flags
            .iter()
            .skip(1)
            .all(|&recomputed| !recomputed)
    );
    assert_eq!(cache.recompute_count(), 1);

    let table = Table::builder(&data)
        .columns(vec![col_name()])
        .state(state)
        .build();
    let (_order, recomputed) = table.tanstack_sorted_flat_row_order_with_cache(2, &mut cache);
    assert!(recomputed);
    assert_eq!(cache.recompute_count(), 2);
}
