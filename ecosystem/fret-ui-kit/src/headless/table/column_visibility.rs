use std::collections::HashMap;

use super::{ColumnDef, ColumnId};

/// TanStack-compatible column visibility map: `column_id -> visible`.
///
/// Columns default to visible when absent from the map.
pub type ColumnVisibilityState = HashMap<ColumnId, bool>;

pub fn is_column_visible(visibility: &ColumnVisibilityState, column: &ColumnId) -> bool {
    visibility.get(column).copied().unwrap_or(true)
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
}
