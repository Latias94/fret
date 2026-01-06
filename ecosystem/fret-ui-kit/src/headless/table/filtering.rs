use std::collections::HashMap;
use std::sync::Arc;

use super::{ColumnDef, ColumnId, RowIndex, RowKey, RowModel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnFilter {
    pub column: ColumnId,
    pub value: Arc<str>,
}

pub type ColumnFiltersState = Vec<ColumnFilter>;
pub type GlobalFilterState = Option<Arc<str>>;

pub fn filter_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    columns: &[ColumnDef<TData>],
    column_filters: &[ColumnFilter],
    global_filter: GlobalFilterState,
) -> RowModel<'a, TData> {
    if row_model.root_rows().is_empty() {
        return row_model.clone();
    }
    if column_filters.is_empty() && global_filter.is_none() {
        return row_model.clone();
    }

    let filter_by_id: HashMap<&str, &super::FilterFn<TData>> = columns
        .iter()
        .filter_map(|c| c.filter_fn.as_ref().map(|f| (c.id.as_ref(), f)))
        .collect();

    fn matches_column_filters<TData>(
        row: &super::Row<'_, TData>,
        filter_by_id: &HashMap<&str, &super::FilterFn<TData>>,
        column_filters: &[ColumnFilter],
    ) -> bool {
        for spec in column_filters {
            let Some(filter_fn) = filter_by_id.get(spec.column.as_ref()).copied() else {
                continue;
            };
            if !filter_fn(row.original, spec.value.as_ref()) {
                return false;
            }
        }
        true
    }

    fn matches_global_filter<TData>(
        row: &super::Row<'_, TData>,
        filter_by_id: &HashMap<&str, &super::FilterFn<TData>>,
        global_filter: &Arc<str>,
    ) -> bool {
        for filter_fn in filter_by_id.values() {
            if filter_fn(row.original, global_filter.as_ref()) {
                return true;
            }
        }
        false
    }

    fn include_row<TData>(
        row: &super::Row<'_, TData>,
        filter_by_id: &HashMap<&str, &super::FilterFn<TData>>,
        column_filters: &[ColumnFilter],
        global_filter: &GlobalFilterState,
    ) -> bool {
        if !matches_column_filters(row, filter_by_id, column_filters) {
            return false;
        }
        let Some(global) = global_filter.as_ref() else {
            return true;
        };
        matches_global_filter(row, filter_by_id, global)
    }

    let mut out_root_rows: Vec<RowIndex> = Vec::new();
    let mut out_flat_rows: Vec<RowIndex> = Vec::new();
    let mut out_rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut out_arena: Vec<super::Row<'a, TData>> = Vec::new();

    fn recurse<'a, TData>(
        source: &RowModel<'a, TData>,
        filter_by_id: &HashMap<&str, &super::FilterFn<TData>>,
        column_filters: &[ColumnFilter],
        global_filter: &GlobalFilterState,
        original: RowIndex,
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_arena: &mut Vec<super::Row<'a, TData>>,
    ) -> Option<RowIndex> {
        let row = source.row(original)?;

        let mut included_children: Vec<RowIndex> = Vec::new();
        for child in &row.sub_rows {
            if let Some(child_new) = recurse(
                source,
                filter_by_id,
                column_filters,
                global_filter,
                *child,
                out_flat_rows,
                out_rows_by_key,
                out_arena,
            ) {
                included_children.push(child_new);
            }
        }

        let self_matches = include_row(row, filter_by_id, column_filters, global_filter);
        let should_include = self_matches || !included_children.is_empty();
        if !should_include {
            return None;
        }

        let new_index = out_arena.len();
        out_arena.push(super::Row {
            key: row.key,
            original: row.original,
            index: row.index,
            depth: row.depth,
            parent: None,
            parent_key: None,
            sub_rows: Vec::new(),
        });
        out_flat_rows.push(new_index);
        out_rows_by_key.insert(row.key, new_index);

        for &child in &included_children {
            if let Some(child_row) = out_arena.get_mut(child) {
                child_row.parent = Some(new_index);
                child_row.parent_key = Some(row.key);
            }
        }
        if let Some(new_row) = out_arena.get_mut(new_index) {
            new_row.sub_rows = included_children;
        }

        Some(new_index)
    }

    for &root in row_model.root_rows() {
        if let Some(new_root) = recurse(
            row_model,
            &filter_by_id,
            column_filters,
            &global_filter,
            root,
            &mut out_flat_rows,
            &mut out_rows_by_key,
            &mut out_arena,
        ) {
            out_root_rows.push(new_root);
        }
    }

    RowModel {
        root_rows: out_root_rows,
        flat_rows: out_flat_rows,
        rows_by_key: out_rows_by_key,
        arena: out_arena,
    }
}

pub fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }

    let needle = needle.as_bytes();
    let hay = haystack.as_bytes();

    for start in 0..=hay.len().saturating_sub(needle.len()) {
        if needle
            .iter()
            .enumerate()
            .all(|(i, &b)| hay[start + i].to_ascii_lowercase() == b.to_ascii_lowercase())
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::table::{Table, create_column_helper};

    #[derive(Debug, Clone)]
    struct Item {
        name: Arc<str>,
        role: Arc<str>,
    }

    #[test]
    fn filter_row_model_applies_column_filters_and_keeps_stable_keys() {
        let data = vec![
            Item {
                name: "a".into(),
                role: "Admin".into(),
            },
            Item {
                name: "b".into(),
                role: "Member".into(),
            },
        ];

        let helper = create_column_helper::<Item>();
        let columns = vec![
            helper
                .clone()
                .accessor("name", |it| it.name.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.name.as_ref(), q)),
            helper
                .accessor("role", |it| it.role.clone())
                .filter_by(|it, q| contains_ascii_case_insensitive(it.role.as_ref(), q)),
        ];

        let table = Table::builder(&data).columns(columns).build();
        let core = table.core_row_model();

        let filtered = filter_row_model(
            core,
            table.columns(),
            &[ColumnFilter {
                column: "role".into(),
                value: "Admin".into(),
            }],
            None,
        );

        assert_eq!(filtered.root_rows().len(), 1);
        let row = filtered.row(filtered.root_rows()[0]).expect("row");
        assert_eq!(row.key, crate::headless::table::RowKey::from_index(0));
        assert!(filtered.row_by_key(row.key).is_some());
    }
}
