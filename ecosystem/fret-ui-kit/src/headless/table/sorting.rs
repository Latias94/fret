use std::cmp::Ordering;
use std::collections::HashMap;

use super::{ColumnDef, ColumnId, RowIndex, RowKey, RowModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
    pub column: ColumnId,
    pub desc: bool,
}

pub type SortingState = Vec<SortSpec>;

pub fn sort_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    sorting: &[SortSpec],
) -> RowModel<'a, TData> {
    if sorting.is_empty() || row_model.root_rows().is_empty() {
        return row_model.clone();
    }

    let cmp_by_id: HashMap<&str, &super::SortCmpFn<TData>> = columns
        .iter()
        .filter_map(|c| c.sort_cmp.as_ref().map(|cmp| (c.id.as_ref(), cmp)))
        .collect();

    let mut out = row_model.clone();

    fn tiebreaker(a: RowKey, b: RowKey) -> Ordering {
        a.cmp(&b)
    }

    fn cmp_rows<TData>(
        arena: &[super::Row<'_, TData>],
        cmp_by_id: &HashMap<&str, &super::SortCmpFn<TData>>,
        sorting: &[SortSpec],
        a: RowIndex,
        b: RowIndex,
    ) -> Ordering {
        let Some(a_row) = arena.get(a) else {
            return Ordering::Equal;
        };
        let Some(b_row) = arena.get(b) else {
            return Ordering::Equal;
        };

        for spec in sorting {
            let Some(cmp) = cmp_by_id.get(spec.column.as_ref()).copied() else {
                continue;
            };
            let mut ord = cmp(a_row.original, b_row.original);
            if spec.desc {
                ord = ord.reverse();
            }
            if ord != Ordering::Equal {
                return ord;
            }
        }

        tiebreaker(a_row.key, b_row.key)
    }

    fn sort_children<TData>(
        row_model: &mut RowModel<'_, TData>,
        cmp_by_id: &HashMap<&str, &super::SortCmpFn<TData>>,
        sorting: &[SortSpec],
        row: RowIndex,
    ) {
        let Some(children) = row_model.row(row).map(|r| r.sub_rows.clone()) else {
            return;
        };
        let arena = row_model.arena.as_slice();
        let mut sorted = children;
        sorted.sort_by(|&a, &b| cmp_rows(arena, cmp_by_id, sorting, a, b));

        if let Some(r) = row_model.arena.get_mut(row) {
            r.sub_rows = sorted.clone();
        }

        for child in sorted {
            sort_children(row_model, cmp_by_id, sorting, child);
        }
    }

    let arena = out.arena.as_slice();
    out.root_rows
        .sort_by(|&a, &b| cmp_rows(arena, &cmp_by_id, sorting, a, b));
    let roots = out.root_rows.clone();
    for root in roots {
        sort_children(&mut out, &cmp_by_id, sorting, root);
    }

    out.flat_rows.clear();
    fn push_flat<TData>(row_model: &mut RowModel<'_, TData>, row: RowIndex) {
        row_model.flat_rows.push(row);
        let Some(r) = row_model.row(row) else {
            return;
        };
        let children = r.sub_rows.clone();
        for child in children {
            push_flat(row_model, child);
        }
    }
    let roots = out.root_rows.clone();
    for root in roots {
        push_flat(&mut out, root);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::table::{Table, create_column_helper};

    #[derive(Debug, Clone)]
    struct Item {
        value: i32,
    }

    #[test]
    fn sort_row_model_sorts_root_rows_by_spec() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let sorted = sort_row_model(core, &columns, &sorting);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![1, 0, 2]);
    }

    #[test]
    fn sort_row_model_respects_descending() {
        let data = vec![Item { value: 2 }, Item { value: 1 }, Item { value: 3 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: true,
        }];

        let sorted = sort_row_model(core, &columns, &sorting);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![2, 0, 1]);
    }

    #[test]
    fn sort_row_model_uses_row_key_tiebreaker_for_determinism() {
        let data = vec![Item { value: 1 }, Item { value: 1 }, Item { value: 1 }];
        let table = Table::builder(&data).build();
        let core = table.core_row_model();

        let helper = create_column_helper::<Item>();
        let columns = vec![helper.accessor("value", |it| it.value)];
        let sorting = vec![SortSpec {
            column: "value".into(),
            desc: false,
        }];

        let sorted = sort_row_model(core, &columns, &sorting);
        let ids = sorted
            .root_rows()
            .iter()
            .filter_map(|&i| sorted.row(i).map(|r| r.key.0))
            .collect::<Vec<_>>();

        assert_eq!(ids, vec![0, 1, 2]);
    }
}
