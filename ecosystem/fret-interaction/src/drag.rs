//! Drag gesture helpers (thresholding and deltas).
//!
//! This is a deliberately small primitive that can be reused by:
//! - in-window floating windows (`imui`),
//! - canvas-space node graph interactions,
//! - docking/multi-window drag choreography.

use fret_core::{Point, Px};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragThreshold {
    pub px: Px,
}

impl DragThreshold {
    pub fn new(px: Px) -> Self {
        Self { px }
    }

    pub fn distance_sq_exceeded(self, start: Point, current: Point) -> bool {
        let dx = current.x.0 - start.x.0;
        let dy = current.y.0 - start.y.0;
        let threshold = self.px.0.max(0.0);
        let threshold_sq = threshold * threshold;
        (dx * dx + dy * dy) >= threshold_sq
    }
}

impl Default for DragThreshold {
    fn default() -> Self {
        Self { px: Px(6.0) }
    }
}
