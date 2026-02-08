use std::collections::HashMap;
use std::sync::Arc;

use super::memo::Memo;
use super::{
    ColumnDef, FilterFnDef, FilteringFnSpec, GlobalFilterState, Row, RowIndex, RowKey, RowModel,
    SortSpec, SortingFnDef, TableOptions, filter_row_model, sort_row_model,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TanStackSortedFlatRowOrderDeps {
    pub items_revision: u64,
    pub data_len: usize,
    pub sorting: Vec<SortSpec>,
    pub column_filters: super::ColumnFiltersState,
    pub global_filter: GlobalFilterState,
    pub options: TableOptions,
    pub global_filter_fn: FilteringFnSpec,
    pub has_get_column_can_global_filter: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlatRowOrderEntry {
    pub index: usize,
    pub key: RowKey,
}

#[derive(Default)]
pub struct TanStackSortedFlatRowOrderCache {
    memo: Memo<(u64, TanStackSortedFlatRowOrderDeps), Arc<[FlatRowOrderEntry]>>,
    columns_signature: u64,
    recompute_count: u64,
}

impl TanStackSortedFlatRowOrderCache {
    pub fn recompute_count(&self) -> u64 {
        self.recompute_count
    }

    /// Returns a stable, memoized ordering of the root row list after filtering + sorting.
    ///
    /// Notes:
    /// - This cache is designed for “rebuild every frame” callers. Keep it outside the ephemeral
    ///   table instance and feed dependency snapshots.
    /// - Dependency tracking is explicit via `deps`. If you change any inputs that are not
    ///   represented in `deps` (e.g. `filter_fns`, `sorting_fns`, or the closure identities),
    ///   you must reset the cache (or bump a revision captured in `deps`).
    pub fn sorted_order<'a, TData>(
        &mut self,
        data: &'a [TData],
        columns: &[ColumnDef<TData>],
        get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
        filter_fns: &HashMap<Arc<str>, FilterFnDef>,
        sorting_fns: &HashMap<Arc<str>, SortingFnDef<TData>>,
        get_column_can_global_filter: Option<&dyn Fn(&ColumnDef<TData>, &TData) -> bool>,
        deps: TanStackSortedFlatRowOrderDeps,
    ) -> (&Arc<[FlatRowOrderEntry]>, bool) {
        debug_assert_eq!(deps.data_len, data.len());
        debug_assert_eq!(
            deps.has_get_column_can_global_filter,
            get_column_can_global_filter.is_some()
        );

        let signature = columns_signature(columns);
        if signature != self.columns_signature {
            self.columns_signature = signature;
            self.memo.reset();
        }

        let sig_and_deps = (signature, deps.clone());
        let (value, recomputed) = self.memo.get_or_compute(sig_and_deps, || {
            compute_sorted_order(
                data,
                columns,
                get_row_key,
                filter_fns,
                sorting_fns,
                get_column_can_global_filter,
                &deps,
            )
        });
        if recomputed {
            self.recompute_count = self.recompute_count.saturating_add(1);
        }
        (value, recomputed)
    }
}

fn columns_signature<TData>(columns: &[ColumnDef<TData>]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    columns.len().hash(&mut hasher);
    for col in columns {
        col.id.as_ref().hash(&mut hasher);
        col.sort_cmp.is_some().hash(&mut hasher);
        col.sorting_fn.hash(&mut hasher);
        col.sort_value.is_some().hash(&mut hasher);
        col.sort_undefined.hash(&mut hasher);
        col.sort_is_undefined.is_some().hash(&mut hasher);
        col.filtering_fn.hash(&mut hasher);
        col.filter_fn.is_some().hash(&mut hasher);
        col.enable_column_filter.hash(&mut hasher);
        col.enable_global_filter.hash(&mut hasher);
        col.invert_sorting.hash(&mut hasher);
        col.sort_desc_first.hash(&mut hasher);
    }
    hasher.finish()
}

fn build_flat_core_row_model<'a, TData>(
    data: &'a [TData],
    get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
) -> RowModel<'a, TData> {
    let mut root_rows: Vec<RowIndex> = Vec::with_capacity(data.len());
    let mut flat_rows: Vec<RowIndex> = Vec::with_capacity(data.len());
    let mut rows_by_key: HashMap<RowKey, RowIndex> = HashMap::with_capacity(data.len());
    let mut rows_by_id: HashMap<super::RowId, RowIndex> = HashMap::with_capacity(data.len());
    let mut arena: Vec<Row<'a, TData>> = Vec::with_capacity(data.len());

    for (index, original) in data.iter().enumerate() {
        let key = get_row_key(original, index, None);
        let id = super::RowId(Arc::<str>::from(key.0.to_string()));
        let row_index = arena.len();
        arena.push(Row {
            id: id.clone(),
            key,
            original,
            index,
            depth: 0,
            parent: None,
            parent_key: None,
            sub_rows: Vec::new(),
        });
        root_rows.push(row_index);
        flat_rows.push(row_index);
        rows_by_key.insert(key, row_index);
        rows_by_id.insert(id, row_index);
    }

    RowModel {
        root_rows,
        flat_rows,
        rows_by_key,
        rows_by_id,
        arena,
    }
}

fn compute_sorted_order<'a, TData>(
    data: &'a [TData],
    columns: &[ColumnDef<TData>],
    get_row_key: &dyn Fn(&TData, usize, Option<&RowKey>) -> RowKey,
    filter_fns: &HashMap<Arc<str>, FilterFnDef>,
    sorting_fns: &HashMap<Arc<str>, SortingFnDef<TData>>,
    get_column_can_global_filter: Option<&dyn Fn(&ColumnDef<TData>, &TData) -> bool>,
    deps: &TanStackSortedFlatRowOrderDeps,
) -> Arc<[FlatRowOrderEntry]> {
    let core = build_flat_core_row_model(data, get_row_key);

    let filtered = if deps.options.manual_filtering {
        core
    } else {
        filter_row_model(
            &core,
            columns,
            &deps.column_filters,
            deps.global_filter.clone(),
            deps.options,
            filter_fns,
            &deps.global_filter_fn,
            get_column_can_global_filter,
        )
    };

    let sorted = if deps.options.manual_sorting {
        filtered
    } else {
        sort_row_model(&filtered, columns, &deps.sorting, sorting_fns)
    };

    let mut out: Vec<FlatRowOrderEntry> = Vec::with_capacity(sorted.root_rows().len());
    for &i in sorted.root_rows() {
        let Some(r) = sorted.row(i) else {
            continue;
        };
        out.push(FlatRowOrderEntry {
            index: r.index,
            key: r.key,
        });
    }
    Arc::from(out.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::{
        FlatRowOrderEntry, TanStackSortedFlatRowOrderCache, TanStackSortedFlatRowOrderDeps,
    };
    use crate::table::{ColumnDef, FilteringFnSpec, RowKey, TableOptions};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    #[derive(Debug)]
    struct Row {
        id: u64,
        name: &'static str,
    }

    fn col_name() -> ColumnDef<Row> {
        ColumnDef::<Row>::new("name")
            .sort_value_by(|row: &Row| {
                crate::table::TanStackValue::String(Arc::<str>::from(row.name))
            })
            .sorting_fn_auto()
            .filtering_fn_auto()
    }

    fn deps_for(
        data: &[Row],
        sorting: Vec<crate::table::SortSpec>,
        column_filters: crate::table::ColumnFiltersState,
        global_filter: crate::table::GlobalFilterState,
    ) -> TanStackSortedFlatRowOrderDeps {
        TanStackSortedFlatRowOrderDeps {
            items_revision: 1,
            data_len: data.len(),
            sorting,
            column_filters,
            global_filter,
            options: TableOptions::default(),
            global_filter_fn: FilteringFnSpec::Auto,
            has_get_column_can_global_filter: false,
        }
    }

    #[test]
    fn sorted_flat_row_order_cache_is_stable_when_deps_unchanged() {
        let data = [Row { id: 2, name: "b" }, Row { id: 1, name: "a" }];
        let columns = vec![col_name()];

        let mut cache = TanStackSortedFlatRowOrderCache::default();
        let filter_fns = HashMap::new();
        let sorting_fns = HashMap::new();

        let deps = deps_for(
            &data,
            vec![crate::table::SortSpec {
                column: "name".into(),
                desc: false,
            }],
            Vec::new(),
            None,
        );

        let (order1, recomputed1) = {
            let (order, recomputed) = cache.sorted_order(
                &data,
                &columns,
                &|row: &Row, _idx, _parent| RowKey(row.id),
                &filter_fns,
                &sorting_fns,
                None,
                deps.clone(),
            );
            (order.clone(), recomputed)
        };
        assert!(recomputed1);
        assert_eq!(cache.recompute_count(), 1);
        assert_eq!(
            &*order1,
            &[
                FlatRowOrderEntry {
                    index: 1,
                    key: RowKey(1)
                },
                FlatRowOrderEntry {
                    index: 0,
                    key: RowKey(2)
                },
            ]
        );

        let (order2, recomputed2) = {
            let (order, recomputed) = cache.sorted_order(
                &data,
                &columns,
                &|row: &Row, _idx, _parent| RowKey(row.id),
                &filter_fns,
                &sorting_fns,
                None,
                deps,
            );
            (order.clone(), recomputed)
        };
        assert!(!recomputed2);
        assert_eq!(cache.recompute_count(), 1);
        assert!(Arc::ptr_eq(&order1, &order2));
    }

    #[test]
    fn sorted_flat_row_order_cache_recomputes_when_filters_change() {
        let data = [
            Row {
                id: 1,
                name: "alpha",
            },
            Row {
                id: 2,
                name: "beta",
            },
        ];
        let columns = vec![col_name()];

        let mut cache = TanStackSortedFlatRowOrderCache::default();
        let filter_fns = HashMap::new();
        let sorting_fns = HashMap::new();

        let deps1 = deps_for(&data, Vec::new(), Vec::new(), None);
        let (_order1, recomputed1) = cache.sorted_order(
            &data,
            &columns,
            &|row: &Row, _idx, _parent| RowKey(row.id),
            &filter_fns,
            &sorting_fns,
            None,
            deps1,
        );
        assert!(recomputed1);
        assert_eq!(cache.recompute_count(), 1);

        let deps2 = deps_for(&data, Vec::new(), Vec::new(), Some(json!("alp")));
        let (_order2, recomputed2) = cache.sorted_order(
            &data,
            &columns,
            &|row: &Row, _idx, _parent| RowKey(row.id),
            &filter_fns,
            &sorting_fns,
            None,
            deps2,
        );
        assert!(recomputed2);
        assert_eq!(cache.recompute_count(), 2);
    }
}
