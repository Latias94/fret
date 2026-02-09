use std::sync::Arc;

use super::ColumnDef;
use super::RowKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellSnapshot {
    pub id: Arc<str>,
    pub column_id: Arc<str>,
    pub is_grouped: bool,
    pub is_placeholder: bool,
    pub is_aggregated: bool,
}

/// A Rust-native equivalent of TanStack `cell.getContext()`.
///
/// This provides the stable ids/keys required to render a cell without requiring consumers to
/// re-derive identity from strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellContextSnapshot {
    pub id: Arc<str>,
    pub row_id: super::RowId,
    pub row_key: RowKey,
    pub column_id: Arc<str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowCellsSnapshot {
    pub all: Vec<CellSnapshot>,
    pub visible: Vec<CellSnapshot>,
    pub left: Vec<CellSnapshot>,
    pub center: Vec<CellSnapshot>,
    pub right: Vec<CellSnapshot>,
}

pub fn snapshot_cells_for_row<TData>(
    row_id: &str,
    all_leaf_columns: &[&ColumnDef<TData>],
    left_leaf_columns: &[&ColumnDef<TData>],
    center_leaf_columns: &[&ColumnDef<TData>],
    right_leaf_columns: &[&ColumnDef<TData>],
    grouped_column_ids: &[Arc<str>],
    row_grouping_column_id: Option<&str>,
    row_has_sub_rows: bool,
) -> RowCellsSnapshot {
    let mk = |col: &ColumnDef<TData>| {
        let column_id = col.id.clone();
        let column_id_ref = column_id.as_ref();

        let column_is_grouped = grouped_column_ids
            .iter()
            .any(|grouped| grouped.as_ref() == column_id_ref);
        let is_grouped = column_is_grouped
            && row_grouping_column_id
                .is_some_and(|grouping_column| grouping_column == column_id_ref);
        let is_placeholder = column_is_grouped && !is_grouped;
        let is_aggregated = !is_grouped && !is_placeholder && row_has_sub_rows;

        CellSnapshot {
            id: Arc::<str>::from(format!("{}_{}", row_id, column_id_ref)),
            column_id,
            is_grouped,
            is_placeholder,
            is_aggregated,
        }
    };

    let left: Vec<CellSnapshot> = left_leaf_columns.iter().copied().map(mk).collect();
    let center: Vec<CellSnapshot> = center_leaf_columns.iter().copied().map(mk).collect();
    let right: Vec<CellSnapshot> = right_leaf_columns.iter().copied().map(mk).collect();

    let mut visible = Vec::new();
    visible.extend(left.iter().cloned());
    visible.extend(center.iter().cloned());
    visible.extend(right.iter().cloned());

    RowCellsSnapshot {
        all: all_leaf_columns.iter().copied().map(mk).collect(),
        visible,
        left,
        center,
        right,
    }
}
