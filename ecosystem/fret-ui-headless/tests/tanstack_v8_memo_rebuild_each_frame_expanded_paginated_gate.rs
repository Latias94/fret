use fret_ui_headless::table::{
    PaginationState, RowKey, Table, TableOptions, TableState, TanStackUngroupedRowModelOrderCache,
};

#[derive(Debug)]
struct Node {
    id: u64,
    children: Vec<Node>,
}

fn node(id: u64, children: Vec<Node>) -> Node {
    Node { id, children }
}

#[test]
fn tanstack_v8_memo_rebuild_each_frame_ungrouped_row_model_order_cache_is_reused_for_expanded_paginated_rows()
 {
    let data = vec![
        node(1, vec![node(10, vec![]), node(11, vec![])]),
        node(2, vec![node(20, vec![])]),
        node(3, vec![]),
    ];

    let mut state = TableState::default();
    state.expanding = [RowKey(1)].into_iter().collect();
    state.pagination = PaginationState {
        page_index: 0,
        page_size: 1,
    };

    let mut cache = TanStackUngroupedRowModelOrderCache::default();

    // paginateExpandedRows=false produces a visible `rows` list that expands within the paginated
    // parent page (and can produce duplicated `flatRows` entries).
    let options = TableOptions {
        paginate_expanded_rows: false,
        ..Default::default()
    };

    let mut recompute_flags = Vec::new();
    for _frame in 0..8 {
        let table = Table::builder(&data)
            .get_row_key(|n, _i, _p| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .state(state.clone())
            .options(options)
            .build();

        let (snapshot, recomputed) = table
            .tanstack_ungrouped_row_model_order_with_cache(1, &mut cache)
            .expect("ungrouped cache is available");
        recompute_flags.push(recomputed);

        let rows = snapshot
            .rows
            .as_ref()
            .iter()
            .map(|k| k.0)
            .collect::<Vec<_>>();
        assert_eq!(rows, vec![1, 10, 11]);
    }

    assert_eq!(recompute_flags.first().copied(), Some(true));
    assert!(recompute_flags.iter().skip(1).all(|&r| !r));
    assert_eq!(cache.recompute_count(), 1);

    // Changing only items_revision invalidates the cached order.
    let table = Table::builder(&data)
        .get_row_key(|n, _i, _p| RowKey(n.id))
        .get_sub_rows(|n, _i| Some(n.children.as_slice()))
        .state(state.clone())
        .options(options)
        .build();
    let (_snapshot, recomputed) = table
        .tanstack_ungrouped_row_model_order_with_cache(2, &mut cache)
        .expect("ungrouped cache is available");
    assert!(recomputed);
    assert_eq!(cache.recompute_count(), 2);
}
