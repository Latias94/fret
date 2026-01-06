use std::collections::{HashMap, HashSet};

use super::{RowIndex, RowKey, RowModel};

/// Selected rows keyed by [`RowKey`].
pub type RowSelectionState = HashSet<RowKey>;

pub fn is_row_selected(row_key: RowKey, selection: &RowSelectionState) -> bool {
    selection.contains(&row_key)
}

/// TanStack-compatible `selectRowsFn`: returns a [`RowModel`] containing only selected rows in the
/// `rows` tree, while keeping `flat_rows` and `rows_by_key` for all selected rows discovered during
/// traversal (including selected sub-rows whose parents are not selected).
pub fn select_rows_fn<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
) -> RowModel<'a, TData> {
    let mut out_root_rows: Vec<RowIndex> = Vec::new();
    let mut out_flat_rows: Vec<RowIndex> = Vec::new();
    let mut out_rows_by_key: HashMap<RowKey, RowIndex> = HashMap::new();
    let mut out_arena: Vec<super::Row<'a, TData>> = Vec::new();

    fn recurse<'a, TData>(
        source: &RowModel<'a, TData>,
        selection: &RowSelectionState,
        original: RowIndex,
        out_root_rows: &mut Vec<RowIndex>,
        out_flat_rows: &mut Vec<RowIndex>,
        out_rows_by_key: &mut HashMap<RowKey, RowIndex>,
        out_arena: &mut Vec<super::Row<'a, TData>>,
        parent_new: Option<RowIndex>,
        is_root: bool,
    ) -> Option<RowIndex> {
        let row = source.row(original)?;
        let selected = is_row_selected(row.key, selection);

        if selected {
            let new_index = out_arena.len();
            out_arena.push(super::Row {
                key: row.key,
                original: row.original,
                index: row.index,
                depth: row.depth,
                parent: parent_new,
                parent_key: row.parent_key,
                sub_rows: Vec::new(),
            });
            out_flat_rows.push(new_index);
            out_rows_by_key.insert(row.key, new_index);
            if is_root {
                out_root_rows.push(new_index);
            }

            let mut selected_children: Vec<RowIndex> = Vec::new();
            for child in &row.sub_rows {
                if let Some(child_new) = recurse(
                    source,
                    selection,
                    *child,
                    out_root_rows,
                    out_flat_rows,
                    out_rows_by_key,
                    out_arena,
                    Some(new_index),
                    false,
                ) {
                    selected_children.push(child_new);
                }
            }
            if let Some(new_row) = out_arena.get_mut(new_index) {
                new_row.sub_rows = selected_children;
            }
            Some(new_index)
        } else {
            for child in &row.sub_rows {
                let _ = recurse(
                    source,
                    selection,
                    *child,
                    out_root_rows,
                    out_flat_rows,
                    out_rows_by_key,
                    out_arena,
                    None,
                    false,
                );
            }
            None
        }
    }

    for &root in row_model.root_rows() {
        let _ = recurse(
            row_model,
            selection,
            root,
            &mut out_root_rows,
            &mut out_flat_rows,
            &mut out_rows_by_key,
            &mut out_arena,
            None,
            true,
        );
    }

    RowModel {
        root_rows: out_root_rows,
        flat_rows: out_flat_rows,
        rows_by_key: out_rows_by_key,
        arena: out_arena,
    }
}

#[cfg(test)]
mod tests {
    use super::super::Table;
    use super::*;

    #[derive(Debug, Clone)]
    struct Person {
        #[allow(dead_code)]
        name: String,
        sub_rows: Option<Vec<Person>>,
    }

    fn make_people(rows: usize, sub_rows: usize) -> Vec<Person> {
        (0..rows)
            .map(|i| Person {
                name: format!("Person {i}"),
                sub_rows: (sub_rows > 0).then(|| {
                    (0..sub_rows)
                        .map(|j| Person {
                            name: format!("Person {i}.{j}"),
                            sub_rows: None,
                        })
                        .collect()
                }),
            })
            .collect()
    }

    #[test]
    fn select_rows_fn_returns_only_selected_rows_in_tree() {
        let data = make_people(5, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        let selection: RowSelectionState = [RowKey::from_index(0), RowKey::from_index(2)]
            .into_iter()
            .collect();

        let selected = select_rows_fn(model, &selection);

        assert_eq!(selected.root_rows().len(), 2);
        assert_eq!(selected.flat_rows().len(), 2);
        assert!(selected.row_by_key(RowKey::from_index(0)).is_some());
        assert!(selected.row_by_key(RowKey::from_index(2)).is_some());
    }

    #[test]
    fn select_rows_fn_recurses_and_filters_sub_rows() {
        let data = make_people(3, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        let root_0 = model.row(model.root_rows()[0]).expect("root row 0");
        let child_0_key = model
            .row(root_0.sub_rows[0])
            .expect("root row 0 child 0")
            .key;
        let selection: RowSelectionState = [root_0.key, child_0_key].into_iter().collect();

        let selected = select_rows_fn(model, &selection);

        let root_0 = selected.row(selected.root_rows()[0]).expect("root row 0");
        assert_eq!(root_0.sub_rows.len(), 1);
        assert_eq!(selected.flat_rows().len(), 2);
        assert!(selected.row_by_key(root_0.key).is_some());
        assert!(selected.row_by_key(child_0_key).is_some());
    }

    #[test]
    fn select_rows_fn_returns_empty_when_no_rows_selected() {
        let data = make_people(5, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        let selection: RowSelectionState = RowSelectionState::default();
        let selected = select_rows_fn(model, &selection);

        assert_eq!(selected.root_rows().len(), 0);
        assert_eq!(selected.flat_rows().len(), 0);
        assert_eq!(selected.arena().len(), 0);
        assert!(selected.rows_by_key().is_empty());
    }
}
