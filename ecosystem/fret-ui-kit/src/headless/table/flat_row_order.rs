use std::sync::Arc;

use super::{ColumnDef, SortCmpFn, SortSpec};

use super::memo::Memo;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FlatRowOrderDeps {
    pub items_revision: u64,
    pub data_len: usize,
    pub sorting: Vec<SortSpec>,
}

#[derive(Default)]
pub struct FlatRowOrderCache {
    memo: Memo<FlatRowOrderDeps, Arc<[usize]>>,
}

impl FlatRowOrderCache {
    pub fn row_order<TData>(
        &mut self,
        data: &[TData],
        columns: &[ColumnDef<TData>],
        deps: FlatRowOrderDeps,
    ) -> (&Arc<[usize]>, bool) {
        let sorting = deps.sorting.clone();
        self.memo
            .get_or_compute(deps, || compute_flat_row_order(data, columns, &sorting))
    }
}

pub fn compute_flat_row_order<TData>(
    data: &[TData],
    columns: &[ColumnDef<TData>],
    sorting: &[SortSpec],
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

    let mut order: Vec<usize> = (0..data.len()).collect();
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

#[cfg(test)]
mod tests {
    use super::compute_flat_row_order;
    use crate::headless::table::{ColumnDef, SortSpec};
    use std::cmp::Ordering;

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
        let order = compute_flat_row_order(&data, &columns, &[]);
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
        );
        assert_eq!(&*order, &[0, 1, 2]);
    }
}
