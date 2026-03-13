//! Alignment guides ("snaplines") for node dragging.
//!
//! This module is UI-light: it computes guide positions and the delta adjustment required to
//! snap a moving rectangle to candidate rectangles.

mod snaplines_align;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SnapGuides {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct SnapResult {
    pub delta_x: f32,
    pub delta_y: f32,
    pub guides: SnapGuides,
}

pub(crate) use snaplines_align::snap_delta_for_rects;

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use super::snap_delta_for_rects;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn snap_delta_for_rects_snaps_left_edge() {
        let moving = rect(10.0, 10.0, 100.0, 50.0);
        let candidates = [rect(50.0, 0.0, 80.0, 40.0)];
        let r = snap_delta_for_rects(moving, &candidates, 0.5);
        assert_eq!(r.delta_x, 0.0);

        let r = snap_delta_for_rects(moving, &candidates, 40.0);
        assert_eq!(r.delta_x, 20.0);
        assert_eq!(r.guides.x, Some(130.0));
    }

    #[test]
    fn snap_delta_for_rects_snaps_center_y() {
        let moving = rect(0.0, 0.0, 10.0, 10.0); // center y = 5
        let candidates = [rect(100.0, 25.0, 10.0, 10.0)]; // center y = 30
        let r = snap_delta_for_rects(moving, &candidates, 30.0);
        assert_eq!(r.delta_y, 25.0);
        assert_eq!(r.guides.y, Some(30.0));
    }
}
