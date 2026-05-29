use std::sync::Arc;

use super::super::super::super::options::TableSortDirection;
use super::super::super::hover::ResponseExt;
use super::TableColumnResizeResponse;

/// Outward response for a single helper-owned table header cell.
#[derive(Debug, Clone)]
pub struct TableHeaderResponse {
    pub(crate) column_index: usize,
    pub(crate) column_id: Option<Arc<str>>,
    pub(crate) sortable: bool,
    pub(crate) sort_direction: Option<TableSortDirection>,
    pub(crate) trigger: ResponseExt,
    pub(crate) resize: TableColumnResizeResponse,
}

impl TableHeaderResponse {
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    pub fn column_id(&self) -> Option<&str> {
        self.column_id.as_deref()
    }

    pub fn sortable(&self) -> bool {
        self.sortable
    }

    pub fn sort_direction(&self) -> Option<TableSortDirection> {
        self.sort_direction
    }

    pub fn response(&self) -> ResponseExt {
        self.trigger
    }

    pub fn clicked(&self) -> bool {
        self.trigger.clicked()
    }

    pub fn activated(&self) -> bool {
        self.trigger.activated()
    }

    pub fn deactivated(&self) -> bool {
        self.trigger.deactivated()
    }

    pub fn resizing(&self) -> bool {
        self.resize.dragging()
    }

    pub fn resize(&self) -> &TableColumnResizeResponse {
        &self.resize
    }
}
