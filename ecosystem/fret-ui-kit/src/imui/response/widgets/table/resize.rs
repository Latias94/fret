use std::sync::Arc;

use fret_core::Px;

use super::super::super::drag::DragResponse;

#[derive(Debug, Clone)]
pub struct TableColumnResizeResponse {
    pub(crate) column_index: usize,
    pub(crate) column_id: Option<Arc<str>>,
    pub(crate) enabled: bool,
    pub(crate) min_width: Option<Px>,
    pub(crate) max_width: Option<Px>,
    pub(crate) drag: DragResponse,
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
