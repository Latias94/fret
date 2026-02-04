use std::collections::HashSet;

use super::{ColumnDef, ColumnId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnPinPosition {
    Left,
    Right,
}

/// TanStack-compatible column pinning state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ColumnPinningState {
    pub left: Vec<ColumnId>,
    pub right: Vec<ColumnId>,
}

pub fn is_column_pinned(
    state: &ColumnPinningState,
    column: &ColumnId,
) -> Option<ColumnPinPosition> {
    if state.left.iter().any(|c| c.as_ref() == column.as_ref()) {
        return Some(ColumnPinPosition::Left);
    }
    if state.right.iter().any(|c| c.as_ref() == column.as_ref()) {
        return Some(ColumnPinPosition::Right);
    }
    None
}

pub fn is_some_columns_pinned(
    state: &ColumnPinningState,
    position: Option<ColumnPinPosition>,
) -> bool {
    match position {
        None => !(state.left.is_empty() && state.right.is_empty()),
        Some(ColumnPinPosition::Left) => !state.left.is_empty(),
        Some(ColumnPinPosition::Right) => !state.right.is_empty(),
    }
}

pub fn pin_column(
    state: &mut ColumnPinningState,
    column: &ColumnId,
    position: Option<ColumnPinPosition>,
) {
    pin_columns(state, position, [column.clone()]);
}

pub fn pin_columns(
    state: &mut ColumnPinningState,
    position: Option<ColumnPinPosition>,
    columns: impl IntoIterator<Item = ColumnId>,
) {
    let mut ids: Vec<ColumnId> = Vec::new();
    let mut id_set: HashSet<ColumnId> = HashSet::new();
    for id in columns {
        if id_set.insert(id.clone()) {
            ids.push(id);
        }
    }

    if id_set.is_empty() {
        return;
    }

    state.left.retain(|c| !id_set.contains(c));
    state.right.retain(|c| !id_set.contains(c));

    match position {
        None => {}
        Some(ColumnPinPosition::Left) => state.left.extend(ids),
        Some(ColumnPinPosition::Right) => state.right.extend(ids),
    }
}

pub fn pinned_column(
    state: &ColumnPinningState,
    column: &ColumnId,
    position: Option<ColumnPinPosition>,
) -> ColumnPinningState {
    let mut next = state.clone();
    pin_column(&mut next, column, position);
    next
}

pub fn pinned_columns(
    state: &ColumnPinningState,
    position: Option<ColumnPinPosition>,
    columns: impl IntoIterator<Item = ColumnId>,
) -> ColumnPinningState {
    let mut next = state.clone();
    pin_columns(&mut next, position, columns);
    next
}

pub fn split_pinned_columns<'c, TData>(
    columns: &[&'c ColumnDef<TData>],
    pinning: &ColumnPinningState,
) -> (
    Vec<&'c ColumnDef<TData>>,
    Vec<&'c ColumnDef<TData>>,
    Vec<&'c ColumnDef<TData>>,
) {
    if columns.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let by_id = columns
        .iter()
        .copied()
        .map(|c| (c.id.as_ref(), c))
        .collect::<std::collections::HashMap<_, _>>();

    let mut out_left: Vec<&ColumnDef<TData>> = Vec::new();
    let mut out_right: Vec<&ColumnDef<TData>> = Vec::new();

    for id in &pinning.left {
        if let Some(col) = by_id.get(id.as_ref()).copied() {
            out_left.push(col);
        }
    }
    for id in &pinning.right {
        if let Some(col) = by_id.get(id.as_ref()).copied() {
            out_right.push(col);
        }
    }

    let pinned: HashSet<&str> = pinning
        .left
        .iter()
        .chain(pinning.right.iter())
        .map(|id| id.as_ref())
        .collect();

    let mut out_center: Vec<&ColumnDef<TData>> = Vec::new();
    for col in columns {
        if pinned.contains(col.id.as_ref()) {
            continue;
        }
        out_center.push(col);
    }

    (out_left, out_center, out_right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_pinned_columns_respects_left_right_and_keeps_center_order() {
        #[derive(Debug)]
        struct Item;

        let a = ColumnDef::<Item>::new("a");
        let b = ColumnDef::<Item>::new("b");
        let c = ColumnDef::<Item>::new("c");
        let d = ColumnDef::<Item>::new("d");

        let columns = vec![&a, &b, &c, &d];
        let pinning = ColumnPinningState {
            left: vec!["c".into()],
            right: vec!["a".into()],
        };

        let (left, center, right) = split_pinned_columns(columns.as_slice(), &pinning);
        assert_eq!(
            left.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>(),
            vec!["c"]
        );
        assert_eq!(
            center.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>(),
            vec!["b", "d"]
        );
        assert_eq!(
            right.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>(),
            vec!["a"]
        );
    }

    #[test]
    fn pin_column_moves_between_sides_and_unpins() {
        let mut state = ColumnPinningState {
            left: vec!["a".into()],
            right: vec!["b".into()],
        };

        pin_column(
            &mut state,
            &ColumnId::from("a"),
            Some(ColumnPinPosition::Right),
        );
        assert!(state.left.is_empty());
        assert_eq!(
            state.right.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["b", "a"]
        );
        assert_eq!(
            is_column_pinned(&state, &ColumnId::from("a")),
            Some(ColumnPinPosition::Right)
        );

        pin_column(&mut state, &ColumnId::from("b"), None);
        assert_eq!(
            state.right.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["a"]
        );
        assert_eq!(is_column_pinned(&state, &ColumnId::from("b")), None);
    }
}
