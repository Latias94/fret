use std::sync::Arc;

use fret_core::Px;

use super::super::super::options::TableSortDirection;
use super::super::drag::DragResponse;
use super::super::hover::ResponseExt;

/// Aggregated response surface for helper-owned table headers.
#[derive(Debug, Clone)]
pub struct TableResponse {
    pub(crate) headers: Vec<TableHeaderResponse>,
}

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

#[derive(Debug, Clone)]
pub struct TableColumnResizeResponse {
    pub(crate) column_index: usize,
    pub(crate) column_id: Option<Arc<str>>,
    pub(crate) enabled: bool,
    pub(crate) min_width: Option<fret_core::Px>,
    pub(crate) max_width: Option<fret_core::Px>,
    pub(crate) drag: DragResponse,
}

impl TableResponse {
    pub fn headers(&self) -> &[TableHeaderResponse] {
        &self.headers
    }

    pub fn header(&self, column_id: &str) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_id.as_deref() == Some(column_id))
    }

    pub fn header_at(&self, column_index: usize) -> Option<&TableHeaderResponse> {
        self.headers
            .iter()
            .find(|header| header.column_index == column_index)
    }
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

impl TableColumnResizeResponse {
    pub fn column_index(&self) -> usize {
        self.column_index
    }

    pub fn column_id(&self) -> Option<&str> {
        self.column_id.as_deref()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn min_width(&self) -> Option<Px> {
        self.min_width
    }

    pub fn max_width(&self) -> Option<Px> {
        self.max_width
    }

    pub fn dragging(&self) -> bool {
        self.drag.dragging
    }

    pub fn drag_started(&self) -> bool {
        self.drag.started
    }

    pub fn drag_stopped(&self) -> bool {
        self.drag.stopped
    }

    pub fn drag_delta_x(&self) -> f32 {
        self.drag.delta.x.0
    }

    pub fn drag_total_x(&self) -> f32 {
        self.drag.total.x.0
    }

    pub fn width_from_start(&self, start_width: Px) -> Px {
        let min = self.min_width.map(|width| width.0).unwrap_or(0.0).max(0.0);
        let max = self.max_width.map(|width| width.0).unwrap_or(f32::INFINITY);
        Px((start_width.0 + self.drag_total_x()).clamp(min, max.max(min)))
    }
}
