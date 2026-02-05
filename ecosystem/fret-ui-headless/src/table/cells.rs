use std::sync::Arc;

use super::ColumnDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellSnapshot {
    pub id: Arc<str>,
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
) -> RowCellsSnapshot {
    let mk = |col: &ColumnDef<TData>| CellSnapshot {
        id: Arc::<str>::from(format!("{}_{}", row_id, col.id.as_ref())),
        column_id: col.id.clone(),
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
