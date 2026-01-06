use std::collections::HashSet;

use super::{RowIndex, RowKey, RowModel};

/// Expanded rows keyed by [`RowKey`].
///
/// This mirrors TanStack Table v8's `ExpandedState` (record keyed by row id), but uses an
/// allocation-free numeric [`RowKey`] in hot paths.
pub type ExpandingState = HashSet<RowKey>;

pub fn is_row_expanded(row_key: RowKey, expanded: &ExpandingState) -> bool {
    expanded.contains(&row_key)
}

pub fn toggle_row_expanded(expanded: &mut ExpandingState, row_key: RowKey, expanded_value: bool) {
    if expanded_value {
        expanded.insert(row_key);
    } else {
        expanded.remove(&row_key);
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
) -> RowModel<'a, TData> {
    if row_model.root_rows().is_empty() {
        return row_model.clone();
    }
    if expanded.is_empty() {
        return row_model.clone();
    }

    let mut out = row_model.clone();
    out.root_rows.clear();

    fn push_visible<TData>(
        source: &RowModel<'_, TData>,
        expanded: &ExpandingState,
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
        if !is_row_expanded(r.key, expanded) {
            return;
        }
        for &child in &r.sub_rows {
            push_visible(source, expanded, out, child);
        }
    }

    for &root in row_model.root_rows() {
        push_visible(row_model, expanded, &mut out.root_rows, root);
    }

    out
}
