use std::sync::Arc;

use super::{ColumnDef, ColumnFilter, GlobalFilterState, SortCmpFn, SortSpec};
use serde_json::Value;

use super::memo::Memo;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FlatRowOrderDeps {
    pub items_revision: u64,
    pub data_len: usize,
    pub sorting: Vec<SortSpec>,
    pub column_filters: Vec<ColumnFilter>,
    pub global_filter: GlobalFilterState,
}

#[derive(Default)]
pub struct FlatRowOrderCache {
    memo: Memo<FlatRowOrderDeps, Arc<[usize]>>,
    columns_signature: u64,
}

impl FlatRowOrderCache {
    pub fn row_order<TData>(
        &mut self,
        data: &[TData],
        columns: &[ColumnDef<TData>],
        deps: FlatRowOrderDeps,
    ) -> (&Arc<[usize]>, bool) {
        let signature = columns_signature(columns);
        if signature != self.columns_signature {
            self.columns_signature = signature;
            self.memo.reset();
        }

        let sorting = deps.sorting.clone();
        let column_filters = deps.column_filters.clone();
        let global_filter = deps.global_filter.clone();
        self.memo.get_or_compute(deps, || {
            compute_flat_row_order(data, columns, &sorting, &column_filters, global_filter)
        })
    }
}

pub fn compute_flat_row_order<TData>(
    data: &[TData],
    columns: &[ColumnDef<TData>],
    sorting: &[SortSpec],
    column_filters: &[ColumnFilter],
    global_filter: GlobalFilterState,
) -> Arc<[usize]> {
    let sorters: Vec<(SortCmpFn<TData>, bool)> = sorting
        .iter()
        .filter_map(|spec| {
            let cmp = columns
                .iter()
                .find(|c| c.id.as_ref() == spec.column.as_ref())?
                .sort_cmp
                .clone()?;
            Some((cmp, spec.desc))
        })
        .collect();

    let resolved_column_filters: Vec<(super::FilterFn<TData>, Value)> = column_filters
        .iter()
        .filter_map(|filter| {
            let filter_fn = columns
                .iter()
                .find(|c| c.id.as_ref() == filter.column.as_ref())?
                .filter_fn
                .clone()?;
            Some((filter_fn, filter.value.clone()))
        })
        .collect();

    let global_filter_fns: Vec<super::FilterFn<TData>> = if global_filter.is_some() {
        columns.iter().filter_map(|c| c.filter_fn.clone()).collect()
    } else {
        Vec::new()
    };

    let mut order: Vec<usize> = (0..data.len())
        .filter(|&i| {
            let row = &data[i];

            for (filter_fn, value) in &resolved_column_filters {
                if !filter_fn(row, value) {
                    return false;
                }
            }

            let Some(global_value) = global_filter.as_ref() else {
                return true;
            };
            for filter_fn in &global_filter_fns {
                if filter_fn(row, &global_value) {
                    return true;
                }
            }
            false
        })
        .collect();
    if !sorters.is_empty() {
        order.sort_by(|&a, &b| {
            let a_row = &data[a];
            let b_row = &data[b];
            for (cmp, desc) in &sorters {
                let mut ord = cmp(a_row, b_row);
                if *desc {
                    ord = ord.reverse();
                }
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            a.cmp(&b)
        });
    }

    Arc::from(order.into_boxed_slice())
}

fn columns_signature<TData>(columns: &[ColumnDef<TData>]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    columns.len().hash(&mut hasher);
    for col in columns {
        col.id.as_ref().hash(&mut hasher);
        col.sort_cmp.is_some().hash(&mut hasher);
        col.filter_fn.is_some().hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::compute_flat_row_order;
    use crate::table::{ColumnDef, SortSpec};
    use std::cmp::Ordering;
    use std::sync::Arc;

    #[derive(Debug)]
    struct Row {
        score: i32,
        name: &'static str,
    }

    fn col<T: 'static>(id: &str, cmp: fn(&T, &T) -> Ordering) -> ColumnDef<T> {
        ColumnDef::new(id).sort_by(cmp)
    }

    #[test]
    fn flat_row_order_is_stable_without_sorting() {
        let data = [
            Row {
                score: 10,
                name: "b",
            },
            Row {
                score: 9,
                name: "a",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(&data, &columns, &[], &[], None);
        assert_eq!(&*order, &[0, 1]);
    }

    #[test]
    fn flat_row_order_sorts_by_single_column() {
        let data = [
            Row {
                score: 10,
                name: "b",
            },
            Row {
                score: 9,
                name: "a",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortSpec {
                column: "score".into(),
                desc: false,
            }],
            &[],
            None,
        );
        assert_eq!(&*order, &[1, 0]);
    }

    #[test]
    fn flat_row_order_sorts_descending() {
        let data = [
            Row {
                score: 10,
                name: "a",
            },
            Row {
                score: 10,
                name: "b",
            },
        ];

        let columns = vec![col::<Row>("name", |a, b| a.name.cmp(b.name))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortSpec {
                column: "name".into(),
                desc: true,
            }],
            &[],
            None,
        );
        assert_eq!(&*order, &[1, 0]);
    }

    #[test]
    fn flat_row_order_uses_index_tiebreaker() {
        let data = [
            Row {
                score: 10,
                name: "x",
            },
            Row {
                score: 10,
                name: "x",
            },
            Row {
                score: 10,
                name: "x",
            },
        ];

        let columns = vec![col::<Row>("score", |a, b| a.score.cmp(&b.score))];
        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortSpec {
                column: "score".into(),
                desc: false,
            }],
            &[],
            None,
        );
        assert_eq!(&*order, &[0, 1, 2]);
    }

    #[test]
    fn flat_row_order_filters_before_sorting() {
        #[derive(Debug)]
        struct Item {
            value: i32,
            kind: Arc<str>,
        }

        let data = [
            Item {
                value: 2,
                kind: "keep".into(),
            },
            Item {
                value: 1,
                kind: "drop".into(),
            },
            Item {
                value: 3,
                kind: "keep".into(),
            },
        ];

        let columns = vec![
            ColumnDef::new("value").sort_by(|a: &Item, b: &Item| a.value.cmp(&b.value)),
            ColumnDef::new("kind").filter_by(|row: &Item, q| row.kind.as_ref() == q),
        ];

        let order = compute_flat_row_order(
            &data,
            &columns,
            &[SortSpec {
                column: "value".into(),
                desc: false,
            }],
            &[crate::table::ColumnFilter {
                column: "kind".into(),
                value: serde_json::Value::from("keep"),
            }],
            None,
        );

        assert_eq!(&*order, &[0, 2]);
    }
}
