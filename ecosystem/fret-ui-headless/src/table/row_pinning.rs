use std::collections::HashSet;

use super::{RowIndex, RowKey, RowModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowPinPosition {
    Top,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowPinningState {
    pub top: Vec<RowKey>,
    pub bottom: Vec<RowKey>,
}

pub fn is_row_pinned(row_key: RowKey, state: &RowPinningState) -> Option<RowPinPosition> {
    if state.top.iter().any(|k| *k == row_key) {
        return Some(RowPinPosition::Top);
    }
    if state.bottom.iter().any(|k| *k == row_key) {
        return Some(RowPinPosition::Bottom);
    }
    None
}

pub fn is_some_rows_pinned(state: &RowPinningState, position: Option<RowPinPosition>) -> bool {
    match position {
        None => !(state.top.is_empty() && state.bottom.is_empty()),
        Some(RowPinPosition::Top) => !state.top.is_empty(),
        Some(RowPinPosition::Bottom) => !state.bottom.is_empty(),
    }
}

pub fn pin_rows(
    state: &mut RowPinningState,
    position: Option<RowPinPosition>,
    rows: impl IntoIterator<Item = RowKey>,
) {
    let mut row_keys: Vec<RowKey> = Vec::new();
    let mut row_key_set: HashSet<RowKey> = HashSet::new();
    for row_key in rows {
        if row_key_set.insert(row_key) {
            row_keys.push(row_key);
        }
    }

    state.top.retain(|k| !row_key_set.contains(k));
    state.bottom.retain(|k| !row_key_set.contains(k));

    match position {
        None => {}
        Some(RowPinPosition::Top) => state.top.extend(row_keys),
        Some(RowPinPosition::Bottom) => state.bottom.extend(row_keys),
    }
}

/// TanStack-compatible helper: pin one row and optionally include its leaf and/or parent rows.
pub fn pin_row<'a, TData>(
    state: &mut RowPinningState,
    position: Option<RowPinPosition>,
    row_model: &RowModel<'a, TData>,
    row_key: RowKey,
    include_leaf_rows: bool,
    include_parent_rows: bool,
) {
    let keys = pin_row_keys(row_model, row_key, include_leaf_rows, include_parent_rows);
    pin_rows(state, position, keys);
}

pub fn pin_row_keys<'a, TData>(
    row_model: &RowModel<'a, TData>,
    row_key: RowKey,
    include_leaf_rows: bool,
    include_parent_rows: bool,
) -> Vec<RowKey> {
    let Some(row_index) = row_model.row_by_key(row_key) else {
        return vec![row_key];
    };

    let mut keys: Vec<RowKey> = Vec::new();

    if include_parent_rows {
        let mut parents_rev: Vec<RowKey> = Vec::new();
        let mut current = row_model.row(row_index);
        while let Some(row) = current {
            let Some(parent) = row.parent else {
                break;
            };
            let Some(parent_row) = row_model.row(parent) else {
                break;
            };
            parents_rev.push(parent_row.key);
            current = Some(parent_row);
        }
        parents_rev.reverse();
        keys.extend(parents_rev);
    }

    keys.push(row_key);

    if include_leaf_rows {
        fn push_descendant_keys<'a, TData>(
            row_model: &RowModel<'a, TData>,
            row: RowIndex,
            out: &mut Vec<RowKey>,
        ) {
            let Some(r) = row_model.row(row) else {
                return;
            };
            for &child in &r.sub_rows {
                let Some(child_row) = row_model.row(child) else {
                    continue;
                };
                out.push(child_row.key);
                push_descendant_keys(row_model, child, out);
            }
        }

        push_descendant_keys(row_model, row_index, &mut keys);
    }

    keys
}

pub fn center_row_keys<'a, TData>(
    visible_root_rows: &[RowIndex],
    row_model: &RowModel<'a, TData>,
    state: &RowPinningState,
) -> Vec<RowKey> {
    let mut pinned = HashSet::<RowKey>::new();
    pinned.extend(state.top.iter().copied());
    pinned.extend(state.bottom.iter().copied());

    visible_root_rows
        .iter()
        .filter_map(|&i| row_model.row(i))
        .map(|r| r.key)
        .filter(|k| !pinned.contains(k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_rows_preserves_input_order_and_dedupes() {
        let mut state = RowPinningState::default();

        pin_rows(
            &mut state,
            Some(RowPinPosition::Top),
            [RowKey(3), RowKey(2), RowKey(3), RowKey(1)],
        );
        assert_eq!(state.top, vec![RowKey(3), RowKey(2), RowKey(1)]);
        assert!(state.bottom.is_empty());

        pin_rows(&mut state, Some(RowPinPosition::Bottom), [RowKey(2)]);
        assert_eq!(state.top, vec![RowKey(3), RowKey(1)]);
        assert_eq!(state.bottom, vec![RowKey(2)]);

        pin_rows(&mut state, None, [RowKey(3)]);
        assert_eq!(state.top, vec![RowKey(1)]);
        assert_eq!(state.bottom, vec![RowKey(2)]);
    }
}
