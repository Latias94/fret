use std::collections::{HashMap, HashSet};

use super::{RowIndex, RowKey, RowModel};

/// Selected rows keyed by [`RowKey`].
pub type RowSelectionState = HashSet<RowKey>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubRowSelection {
    None,
    Some,
    All,
}

pub fn is_row_selected(row_key: RowKey, selection: &RowSelectionState) -> bool {
    selection.contains(&row_key)
}

pub fn is_sub_row_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    row: RowIndex,
) -> SubRowSelection {
    let Some(row) = row_model.row(row) else {
        return SubRowSelection::None;
    };
    if row.sub_rows.is_empty() {
        return SubRowSelection::None;
    }

    let mut all_children_selected = true;
    let mut some_selected = false;

    for &child in &row.sub_rows {
        if some_selected && !all_children_selected {
            break;
        }

        let Some(child_row) = row_model.row(child) else {
            all_children_selected = false;
            continue;
        };

        if is_row_selected(child_row.key, selection) {
            some_selected = true;
        } else {
            all_children_selected = false;
        }

        if !child_row.sub_rows.is_empty() {
            match is_sub_row_selected(row_model, selection, child) {
                SubRowSelection::All => {
                    some_selected = true;
                }
                SubRowSelection::Some => {
                    some_selected = true;
                    all_children_selected = false;
                }
                SubRowSelection::None => {
                    all_children_selected = false;
                }
            }
        }
    }

    if all_children_selected {
        SubRowSelection::All
    } else if some_selected {
        SubRowSelection::Some
    } else {
        SubRowSelection::None
    }
}

pub fn row_is_some_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    row: RowIndex,
) -> bool {
    is_sub_row_selected(row_model, selection, row) == SubRowSelection::Some
}

pub fn row_is_all_sub_rows_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    row: RowIndex,
) -> bool {
    is_sub_row_selected(row_model, selection, row) == SubRowSelection::All
}

pub fn selected_flat_row_count<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
) -> usize {
    row_model
        .flat_rows()
        .iter()
        .filter_map(|&i| row_model.row(i).map(|r| r.key))
        .filter(|k| is_row_selected(*k, selection))
        .count()
}

pub fn selected_root_row_count<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
) -> usize {
    row_model
        .root_rows()
        .iter()
        .filter_map(|&i| row_model.row(i).map(|r| r.key))
        .filter(|k| is_row_selected(*k, selection))
        .count()
}

pub fn is_all_rows_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    enable_row_selection: bool,
) -> bool {
    if row_model.flat_rows().is_empty() {
        return false;
    }
    if selection.is_empty() {
        return false;
    }

    row_model.flat_rows().iter().all(|&i| {
        let Some(row) = row_model.row(i) else {
            return true;
        };
        if !enable_row_selection {
            return true;
        }
        is_row_selected(row.key, selection)
    })
}

pub fn is_some_rows_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
) -> bool {
    let total = row_model.flat_rows().len();
    if total == 0 {
        return false;
    }
    let selected = selected_flat_row_count(row_model, selection);
    selected > 0 && selected < total
}

pub fn toggle_all_rows_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    value: Option<bool>,
    enable_row_selection: bool,
) -> RowSelectionState {
    let mut next = selection.clone();
    let value =
        value.unwrap_or_else(|| !is_all_rows_selected(row_model, selection, enable_row_selection));

    if value {
        for &i in row_model.flat_rows() {
            let Some(row) = row_model.row(i) else {
                continue;
            };
            if enable_row_selection {
                next.insert(row.key);
            }
        }
    } else {
        for &i in row_model.flat_rows() {
            let Some(row) = row_model.row(i) else {
                continue;
            };
            next.remove(&row.key);
        }
    }

    next
}

pub fn toggle_all_page_rows_selected<'a, TData>(
    page_row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    value: Option<bool>,
    enable_row_selection: bool,
) -> RowSelectionState {
    toggle_all_rows_selected(page_row_model, selection, value, enable_row_selection)
}

pub fn toggle_row_selected<'a, TData>(
    row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    row_key: RowKey,
    value: Option<bool>,
    select_children: bool,
    enable_row_selection: bool,
    enable_multi_row_selection: bool,
    enable_sub_row_selection: bool,
) -> RowSelectionState {
    let mut next = selection.clone();

    let current = is_row_selected(row_key, selection);
    let value = value.unwrap_or(!current);

    if value && !enable_row_selection {
        return next;
    }

    if value && !enable_multi_row_selection {
        next.clear();
    }

    if value {
        next.insert(row_key);
    } else {
        next.remove(&row_key);
    }

    if !select_children || !enable_sub_row_selection {
        return next;
    }

    let Some(row_index) = row_model.row_by_key(row_key) else {
        return next;
    };

    let mut stack: Vec<RowIndex> = Vec::new();
    stack.push(row_index);

    while let Some(i) = stack.pop() {
        let Some(r) = row_model.row(i) else {
            continue;
        };
        for &child in &r.sub_rows {
            let Some(child_row) = row_model.row(child) else {
                continue;
            };
            if value {
                next.insert(child_row.key);
            } else {
                next.remove(&child_row.key);
            }
            stack.push(child);
        }
    }

    next
}

/// TanStack `getIsAllPageRowsSelected` semantics.
///
/// Notes:
/// - Only rows that can be selected are considered.
/// - Unlike `getIsAllRowsSelected`, this does not require `rowSelection` to be non-empty.
pub fn is_all_page_rows_selected<'a, TData>(
    page_row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    enable_row_selection: bool,
) -> bool {
    if !enable_row_selection {
        return false;
    }

    let mut any = false;
    for &i in page_row_model.flat_rows() {
        let Some(row) = page_row_model.row(i) else {
            continue;
        };
        any = true;
        if !is_row_selected(row.key, selection) {
            return false;
        }
    }
    any
}

pub fn is_some_page_rows_selected<'a, TData>(
    page_row_model: &RowModel<'a, TData>,
    selection: &RowSelectionState,
    enable_row_selection: bool,
) -> bool {
    if is_all_page_rows_selected(page_row_model, selection, enable_row_selection) {
        return false;
    }
    if !enable_row_selection {
        return false;
    }

    page_row_model.flat_rows().iter().any(|&i| {
        let Some(row) = page_row_model.row(i) else {
            return false;
        };
        is_row_selected(row.key, selection)
    })
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

    #[test]
    fn toggle_all_rows_selected_selects_and_deselects_flat_rows() {
        let data = make_people(5, 0);
        let table = Table::builder(&data).build();
        let model = table.core_row_model();

        let selection = RowSelectionState::default();
        let selection = toggle_all_rows_selected(model, &selection, Some(true), true);
        assert!(is_all_rows_selected(model, &selection, true));

        let selection = toggle_all_rows_selected(model, &selection, Some(false), true);
        assert!(selection.is_empty());
        assert!(!is_some_rows_selected(model, &selection));
    }

    #[test]
    fn sub_row_selection_reports_some_and_all() {
        let data = make_people(2, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        let root = model.root_rows()[0];
        let root_key = model.row(root).unwrap().key;
        let child0 = model.row(root).unwrap().sub_rows[0];
        let child1 = model.row(root).unwrap().sub_rows[1];
        let child0_key = model.row(child0).unwrap().key;
        let child1_key = model.row(child1).unwrap().key;

        let selection: RowSelectionState = [child0_key].into_iter().collect();
        assert_eq!(
            is_sub_row_selected(model, &selection, root),
            SubRowSelection::Some
        );
        assert!(row_is_some_selected(model, &selection, root));
        assert!(!row_is_all_sub_rows_selected(model, &selection, root));
        assert!(!is_row_selected(root_key, &selection));

        let selection: RowSelectionState = [child0_key, child1_key].into_iter().collect();
        assert_eq!(
            is_sub_row_selected(model, &selection, root),
            SubRowSelection::All
        );
        assert!(!row_is_some_selected(model, &selection, root));
        assert!(row_is_all_sub_rows_selected(model, &selection, root));
    }

    #[test]
    fn toggle_row_selected_can_select_children() {
        let data = make_people(1, 2);
        let table = Table::builder(&data)
            .get_sub_rows(|p, _| p.sub_rows.as_deref())
            .build();
        let model = table.core_row_model();

        let root = model.root_rows()[0];
        let root_key = model.row(root).unwrap().key;
        let child0 = model.row(root).unwrap().sub_rows[0];
        let child1 = model.row(root).unwrap().sub_rows[1];
        let child0_key = model.row(child0).unwrap().key;
        let child1_key = model.row(child1).unwrap().key;

        let selection = RowSelectionState::default();
        let selection = toggle_row_selected(
            model,
            &selection,
            root_key,
            Some(true),
            true,
            true,
            true,
            true,
        );
        assert!(is_row_selected(root_key, &selection));
        assert!(is_row_selected(child0_key, &selection));
        assert!(is_row_selected(child1_key, &selection));

        let selection = toggle_row_selected(
            model,
            &selection,
            root_key,
            Some(false),
            true,
            true,
            true,
            true,
        );
        assert!(!is_row_selected(root_key, &selection));
        assert!(!is_row_selected(child0_key, &selection));
        assert!(!is_row_selected(child1_key, &selection));
    }
}
