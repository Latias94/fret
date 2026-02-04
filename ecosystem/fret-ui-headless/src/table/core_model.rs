use std::collections::BTreeMap;
use std::sync::Arc;

use super::{HeaderGroupSnapshot, RowCellsSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnNodeSnapshot {
    pub id: Arc<str>,
    pub depth: usize,
    pub parent_id: Option<Arc<str>>,
    pub child_ids: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafColumnsSnapshot {
    pub all: Vec<Arc<str>>,
    pub visible: Vec<Arc<str>>,
    pub left_visible: Vec<Arc<str>>,
    pub center_visible: Vec<Arc<str>>,
    pub right_visible: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowModelIdSnapshot {
    pub root: Vec<Arc<str>>,
    pub flat: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreRowsSnapshot {
    pub core: RowModelIdSnapshot,
    pub row_model: RowModelIdSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreModelSnapshot {
    pub column_tree: Vec<ColumnNodeSnapshot>,
    pub leaf_columns: LeafColumnsSnapshot,
    pub header_groups: Vec<HeaderGroupSnapshot>,
    pub left_header_groups: Vec<HeaderGroupSnapshot>,
    pub center_header_groups: Vec<HeaderGroupSnapshot>,
    pub right_header_groups: Vec<HeaderGroupSnapshot>,
    pub rows: CoreRowsSnapshot,
    pub cells: BTreeMap<Arc<str>, RowCellsSnapshot>,
}
