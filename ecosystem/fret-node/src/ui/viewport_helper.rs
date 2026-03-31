//! Shared viewport math helpers.
//!
//! App-facing and retained viewport entry points now converge on `NodeGraphController`; this
//! module only keeps the reusable pan-centering math used by those controller-backed flows.

use fret_core::Rect;

use crate::core::CanvasPoint;

pub(super) fn pan_for_center(bounds: Rect, center: CanvasPoint, zoom: f32) -> CanvasPoint {
    let z = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    let w = bounds.size.width.0;
    let h = bounds.size.height.0;

    CanvasPoint {
        x: w / (2.0 * z) - center.x,
        y: h / (2.0 * z) - center.y,
    }
}

#[cfg(test)]
mod tests {
    use fret_core::{Point, Px, Rect, Size};

    use super::pan_for_center;
    use crate::core::CanvasPoint;

    #[test]
    fn pan_for_center_aligns_canvas_point_to_window_center() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );
        let center = CanvasPoint { x: 10.0, y: 20.0 };
        let pan = pan_for_center(bounds, center, 2.0);

        assert!((pan.x - (800.0 / 4.0 - 10.0)).abs() <= 1.0e-6);
        assert!((pan.y - (600.0 / 4.0 - 20.0)).abs() <= 1.0e-6);
    }
}
