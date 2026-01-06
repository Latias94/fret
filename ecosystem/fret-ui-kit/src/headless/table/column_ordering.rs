use std::collections::HashMap;

use super::{ColumnDef, ColumnId};

/// TanStack-compatible column order: an ordered list of column ids.
///
/// Any columns not present in this list keep their original relative order and
/// are appended to the end.
pub type ColumnOrderState = Vec<ColumnId>;

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
}
