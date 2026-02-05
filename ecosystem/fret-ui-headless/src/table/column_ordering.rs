use std::collections::HashMap;

use super::{ColumnDef, ColumnId};

/// TanStack-compatible column order: an ordered list of column ids.
///
/// Any columns not present in this list keep their original relative order and
/// are appended to the end.
pub type ColumnOrderState = Vec<ColumnId>;

pub fn set_column_order(order: &mut ColumnOrderState, ids: impl IntoIterator<Item = ColumnId>) {
    order.clear();
    for id in ids {
        order.push(id);
    }
}

pub fn set_column_order_for<'a>(
    order: &mut ColumnOrderState,
    ids: impl IntoIterator<Item = &'a str>,
) {
    set_column_order(order, ids.into_iter().map(ColumnId::from));
}

pub fn move_column(order: &mut ColumnOrderState, id: &ColumnId, to_index: usize) {
    if order.is_empty() {
        order.push(id.clone());
        return;
    }

    let from = order.iter().position(|c| c.as_ref() == id.as_ref());
    let mut next: Vec<ColumnId> = Vec::with_capacity(order.len() + 1);

    for (idx, c) in order.iter().enumerate() {
        if Some(idx) == from {
            continue;
        }
        next.push(c.clone());
    }

    let insert_at = to_index.min(next.len());
    next.insert(insert_at, id.clone());
    *order = next;
}

pub fn moved_column(order: &ColumnOrderState, id: &ColumnId, to_index: usize) -> ColumnOrderState {
    let mut next = order.clone();
    move_column(&mut next, id, to_index);
    next
}

pub fn order_columns<'c, TData>(
    columns: &'c [ColumnDef<TData>],
    order: &[ColumnId],
) -> Vec<&'c ColumnDef<TData>> {
    if columns.is_empty() {
        return Vec::new();
    }
    if order.is_empty() {
        return columns.iter().collect();
    }

    let mut remaining: HashMap<&str, &ColumnDef<TData>> =
        columns.iter().map(|c| (c.id.as_ref(), c)).collect();

    let mut out: Vec<&ColumnDef<TData>> = Vec::with_capacity(columns.len());
    for id in order {
        if let Some(col) = remaining.remove(id.as_ref()) {
            out.push(col);
        }
    }

    for col in columns {
        if remaining.remove(col.id.as_ref()).is_some() {
            out.push(col);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_columns_respects_explicit_order_then_appends_rest() {
        #[derive(Debug)]
        struct Item;

        let columns = vec![
            ColumnDef::<Item>::new("a"),
            ColumnDef::<Item>::new("b"),
            ColumnDef::<Item>::new("c"),
        ];

        let out = order_columns(&columns, &[ColumnId::from("c"), ColumnId::from("a")]);
        let ids = out.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>();

        assert_eq!(ids, vec!["c", "a", "b"]);
    }

    #[test]
    fn order_columns_ignores_unknown_ids() {
        #[derive(Debug)]
        struct Item;

        let columns = vec![ColumnDef::<Item>::new("a"), ColumnDef::<Item>::new("b")];
        let out = order_columns(&columns, &[ColumnId::from("x"), ColumnId::from("b")]);

        let ids = out.iter().map(|c| c.id.as_ref()).collect::<Vec<_>>();
        assert_eq!(ids, vec!["b", "a"]);
    }

    #[test]
    fn set_column_order_preserves_input_order_including_duplicates() {
        let mut order: ColumnOrderState = vec!["a".into()];
        set_column_order_for(&mut order, ["b", "a", "b"]);
        assert_eq!(
            order.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["b", "a", "b"]
        );
    }

    #[test]
    fn move_column_inserts_when_missing_and_reorders_when_present() {
        let mut order: ColumnOrderState = vec!["a".into(), "b".into(), "c".into()];
        move_column(&mut order, &ColumnId::from("b"), 0);
        assert_eq!(
            order.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["b", "a", "c"]
        );

        move_column(&mut order, &ColumnId::from("x"), 1);
        assert_eq!(
            order.iter().map(|c| c.as_ref()).collect::<Vec<_>>(),
            vec!["b", "x", "a", "c"]
        );
    }
}
