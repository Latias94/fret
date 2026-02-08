use std::collections::BTreeMap;
use std::sync::Arc;

use super::{HeaderGroupSnapshot, RowCellsSnapshot};

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderSizingSnapshot {
    pub size: BTreeMap<Arc<str>, f32>,
    pub start: BTreeMap<Arc<str>, f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnCapabilitySnapshot {
    pub can_hide: bool,
    pub can_pin: bool,
    pub pin_position: Option<super::ColumnPinPosition>,
    pub pinned_index: i32,
    pub can_resize: bool,
    pub is_visible: bool,
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct CoreModelSnapshot {
    pub schema_version: u32,
    pub column_tree: Vec<ColumnNodeSnapshot>,
    pub column_capabilities: BTreeMap<Arc<str>, ColumnCapabilitySnapshot>,
    pub leaf_columns: LeafColumnsSnapshot,
    pub header_groups: Vec<HeaderGroupSnapshot>,
    pub left_header_groups: Vec<HeaderGroupSnapshot>,
    pub center_header_groups: Vec<HeaderGroupSnapshot>,
    pub right_header_groups: Vec<HeaderGroupSnapshot>,
    pub header_sizing: HeaderSizingSnapshot,
    pub rows: CoreRowsSnapshot,
    pub cells: BTreeMap<Arc<str>, RowCellsSnapshot>,
}
