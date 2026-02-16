use std::collections::HashMap;

use super::{ColumnDef, ColumnId};

/// TanStack-compatible column visibility map: `column_id -> visible`.
///
/// Columns default to visible when absent from the map.
pub type ColumnVisibilityState = HashMap<ColumnId, bool>;

pub fn is_column_visible(visibility: &ColumnVisibilityState, column: &ColumnId) -> bool {
    visibility.get(column).copied().unwrap_or(true)
}

pub fn set_column_visible(
    visibility: &mut ColumnVisibilityState,
    column: &ColumnId,
    visible: bool,
) {
    if visible {
        visibility.remove(column);
    } else {
        visibility.insert(column.clone(), false);
    }
}

pub fn toggle_column_visible(
    visibility: &mut ColumnVisibilityState,
    column: &ColumnId,
    visible: Option<bool>,
) {
    let visible = visible.unwrap_or_else(|| !is_column_visible(visibility, column));
    set_column_visible(visibility, column, visible);
}

pub fn toggled_column_visible(
    visibility: &ColumnVisibilityState,
    column: &ColumnId,
    visible: Option<bool>,
) -> ColumnVisibilityState {
    let mut next = visibility.clone();
    toggle_column_visible(&mut next, column, visible);
    next
}

pub fn visible_columns<'c, TData>(
    columns: &'c [&'c ColumnDef<TData>],
    visibility: &ColumnVisibilityState,
) -> Vec<&'c ColumnDef<TData>> {
    columns
        .iter()
        .copied()
        .filter(|c| is_column_visible(visibility, &c.id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_column_visible_defaults_to_true() {
        let visibility = ColumnVisibilityState::default();
        assert!(is_column_visible(&visibility, &ColumnId::from("a")));
    }

    #[test]
    fn visible_columns_filters_by_state() {
        #[derive(Debug)]
        struct Item;

        let a = ColumnDef::<Item>::new("a");
        let b = ColumnDef::<Item>::new("b");
        let c = ColumnDef::<Item>::new("c");

        let columns = vec![&a, &b, &c];
        let mut visibility = ColumnVisibilityState::default();
        visibility.insert(ColumnId::from("b"), false);

        let out = visible_columns(columns.as_slice(), &visibility);
        let ids = out.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>();

        assert_eq!(ids, vec!["a", "c"]);
    }

    #[test]
    fn set_column_visible_removes_true_and_sets_false() {
        let mut visibility = ColumnVisibilityState::default();
        let a = ColumnId::from("a");

        set_column_visible(&mut visibility, &a, false);
        assert_eq!(visibility.get(&a).copied(), Some(false));

        set_column_visible(&mut visibility, &a, true);
        assert!(!visibility.contains_key(&a));
        assert!(is_column_visible(&visibility, &a));
    }

    #[test]
    fn toggle_column_visible_flips_default_visible() {
        let mut visibility = ColumnVisibilityState::default();
        let a = ColumnId::from("a");

        assert!(is_column_visible(&visibility, &a));
        toggle_column_visible(&mut visibility, &a, None);
        assert!(!is_column_visible(&visibility, &a));
        toggle_column_visible(&mut visibility, &a, None);
        assert!(is_column_visible(&visibility, &a));
    }
}
