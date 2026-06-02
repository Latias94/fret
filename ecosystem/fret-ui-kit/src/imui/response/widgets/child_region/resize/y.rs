use fret_core::Px;

use super::super::super::super::drag::DragResponse;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default)]
pub struct ChildRegionResizeYResponse {
    pub(crate) enabled: bool,
    pub(crate) min_height: Option<Px>,
    pub(crate) max_height: Option<Px>,
    pub(crate) drag: DragResponse,
}

impl ChildRegionResizeYResponse {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn min_height(&self) -> Option<Px> {
        self.min_height
    }

    pub fn max_height(&self) -> Option<Px> {
        self.max_height
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

    pub fn drag_delta_y(&self) -> f32 {
        self.drag.delta.y.0
    }

    pub fn drag_total_y(&self) -> f32 {
        self.drag.total.y.0
    }

    pub fn height_from_start(&self, start_height: Px) -> Px {
        let min = self
            .min_height
            .map(|height| height.0)
            .unwrap_or(0.0)
            .max(0.0);
        let max = self
            .max_height
            .map(|height| height.0)
            .unwrap_or(f32::INFINITY);
        Px((start_height.0 + self.drag_total_y()).clamp(min, max.max(min)))
    }
}
