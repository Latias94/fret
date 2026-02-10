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
        if !dx.is_finite() || !dy.is_finite() {
            return false;
        }

        let dist2 = dx * dx + dy * dy;
        let threshold = self.px.0;
        if !threshold.is_finite() || threshold <= 0.0 {
            return dist2 > 0.0;
        }

        dist2 >= threshold * threshold
    }
}

impl Default for DragThreshold {
    fn default() -> Self {
        Self { px: Px(6.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_threshold_matches_imgui_default() {
        assert_eq!(DragThreshold::default().px, Px(6.0));
    }

    #[test]
    fn negative_threshold_is_treated_as_zero() {
        let t = DragThreshold::new(Px(-10.0));
        let start = Point::new(Px(0.0), Px(0.0));
        let current = Point::new(Px(0.0), Px(0.0));
        assert!(!t.distance_sq_exceeded(start, current));

        let moved = Point::new(Px(1.0), Px(0.0));
        assert!(t.distance_sq_exceeded(start, moved));
    }

    #[test]
    fn threshold_exceeded_is_inclusive() {
        let t = DragThreshold::new(Px(2.0));
        let start = Point::new(Px(0.0), Px(0.0));
        let current = Point::new(Px(2.0), Px(0.0));
        assert!(t.distance_sq_exceeded(start, current));
    }
}
