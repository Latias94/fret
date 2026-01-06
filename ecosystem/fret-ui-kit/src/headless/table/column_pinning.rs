use std::collections::HashSet;

use super::{ColumnDef, ColumnId};

/// TanStack-compatible column pinning state.
#[derive(Debug, Clone, Default)]
pub struct ColumnPinningState {
    pub left: Vec<ColumnId>,
    pub right: Vec<ColumnId>,
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
}
