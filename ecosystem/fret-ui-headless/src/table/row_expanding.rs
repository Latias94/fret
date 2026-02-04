use std::collections::HashSet;
use std::iter::FromIterator;

use super::{RowIndex, RowKey, RowModel};

/// TanStack-compatible expanded state.
///
/// In TanStack Table v8, the expanded state is `true | Record<RowId, boolean>`.
/// We model this as:
/// - `All`: every row is considered expanded (`expanded === true`).
/// - `Keys`: a set of explicitly expanded rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpandingState {
    All,
    Keys(HashSet<RowKey>),
}

impl Default for ExpandingState {
    fn default() -> Self {
        Self::Keys(HashSet::new())
    }
}

impl FromIterator<RowKey> for ExpandingState {
    fn from_iter<T: IntoIterator<Item = RowKey>>(iter: T) -> Self {
        Self::Keys(HashSet::from_iter(iter))
    }
}

pub fn is_row_expanded(row_key: RowKey, expanded: &ExpandingState) -> bool {
    match expanded {
        ExpandingState::All => true,
        ExpandingState::Keys(keys) => keys.contains(&row_key),
    }
}

pub fn is_some_rows_expanded(expanded: &ExpandingState) -> bool {
    match expanded {
        ExpandingState::All => true,
        ExpandingState::Keys(keys) => !keys.is_empty(),
    }
}

pub fn set_all_rows_expanded(expanded: &mut ExpandingState, expanded_value: bool) {
    if expanded_value {
        *expanded = ExpandingState::All;
    } else {
        *expanded = ExpandingState::default();
    }
}

pub fn toggle_all_rows_expanded(expanded: &mut ExpandingState, expanded_value: Option<bool>) {
    let next = expanded_value.unwrap_or_else(|| !matches!(expanded, ExpandingState::All));
    set_all_rows_expanded(expanded, next);
}

pub fn toggle_row_expanded<'a, TData>(
    expanded: &mut ExpandingState,
    row_model: &RowModel<'a, TData>,
    row_key: RowKey,
    expanded_value: Option<bool>,
) {
    let exists = is_row_expanded(row_key, expanded);
    let expanded_value = expanded_value.unwrap_or(!exists);

    match expanded {
        ExpandingState::All => {
            if expanded_value {
                return;
            }

            // Convert `All` -> `Keys(all_row_ids)` and then remove the row.
            let mut keys: HashSet<RowKey> = row_model.rows_by_key().keys().copied().collect();
            keys.remove(&row_key);
            *expanded = ExpandingState::Keys(keys);
        }
        ExpandingState::Keys(keys) => {
            if expanded_value {
                keys.insert(row_key);
            } else {
                keys.remove(&row_key);
            }
        }
    }
}

pub fn row_can_expand<TData>(row_model: &RowModel<'_, TData>, row: RowIndex) -> bool {
    row_model.row(row).is_some_and(|r| !r.sub_rows.is_empty())
}

pub fn row_is_all_parents_expanded<TData>(
    row_model: &RowModel<'_, TData>,
    expanded: &ExpandingState,
    row: RowIndex,
) -> bool {
    let mut current = row_model.row(row);
    while let Some(r) = current {
        let Some(parent) = r.parent else {
            return true;
        };
        let Some(parent_row) = row_model.row(parent) else {
            return true;
        };
        if !is_row_expanded(parent_row.key, expanded) {
            return false;
        }
        current = Some(parent_row);
    }
    true
}

pub fn expanded_depth<TData>(row_model: &RowModel<'_, TData>, expanded: &ExpandingState) -> u16 {
    match expanded {
        ExpandingState::All => row_model
            .arena()
            .iter()
            .map(|r| r.depth.saturating_add(1))
            .max()
            .unwrap_or(0),
        ExpandingState::Keys(keys) => keys
            .iter()
            .filter_map(|k| row_model.row_by_key(*k).and_then(|i| row_model.row(i)))
            .map(|r| r.depth.saturating_add(1))
            .max()
            .unwrap_or(0),
    }
}

/// TanStack-aligned "expanded row model": a `RowModel` whose `flat_rows` contain only the
/// visible rows under the current expansion state.
///
/// Notes:
/// - `arena` and `rows_by_key` are preserved (like TanStack's `rowsById`) so callers can still
///   resolve row metadata for collapsed descendants.
/// - `root_rows` are not changed by expansion; only the flattened visible order is.
pub fn expand_row_model<'a, TData>(
    row_model: &RowModel<'a, TData>,
    expanded: &ExpandingState,
    mut row_is_expanded: impl FnMut(RowKey, &TData) -> bool,
) -> RowModel<'a, TData> {
    if row_model.root_rows().is_empty() {
        return row_model.clone();
    }
    if !is_some_rows_expanded(expanded) {
        return row_model.clone();
    }

    let mut out = row_model.clone();
    out.root_rows.clear();

    fn push_visible<TData>(
        source: &RowModel<'_, TData>,
        row_is_expanded: &mut impl FnMut(RowKey, &TData) -> bool,
        out: &mut Vec<RowIndex>,
        row: RowIndex,
    ) {
        out.push(row);
        let Some(r) = source.row(row) else {
            return;
        };
        if r.sub_rows.is_empty() {
            return;
        }
        if !row_is_expanded(r.key, r.original) {
            return;
        }
        for &child in &r.sub_rows {
            push_visible(source, row_is_expanded, out, child);
        }
    }

    for &root in row_model.root_rows() {
        push_visible(row_model, &mut row_is_expanded, &mut out.root_rows, root);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Table;

    #[derive(Debug, Clone)]
    struct Node {
        id: u64,
        children: Vec<Node>,
    }

    #[test]
    fn expanded_depth_tracks_max_depth_plus_one() {
        let data = vec![Node {
            id: 1,
            children: vec![Node {
                id: 10,
                children: vec![Node {
                    id: 100,
                    children: Vec::new(),
                }],
            }],
        }];

        let table = Table::builder(&data)
            .get_row_key(|n, _i, _p| RowKey(n.id))
            .get_sub_rows(|n, _i| Some(n.children.as_slice()))
            .build();
        let core = table.core_row_model();

        let expanded = ExpandingState::from_iter([RowKey(1), RowKey(10)]);
        assert_eq!(expanded_depth(core, &expanded), 2);
    }

    #[test]
    fn toggle_all_sets_all_variant() {
        let mut expanded = ExpandingState::default();
        toggle_all_rows_expanded(&mut expanded, Some(true));
        assert!(matches!(expanded, ExpandingState::All));
        toggle_all_rows_expanded(&mut expanded, Some(false));
        assert!(matches!(expanded, ExpandingState::Keys(_)));
    }
}
