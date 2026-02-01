use fret_core::{Point, Px, Rect, Size};

fn normalize_scale_factor(scale_factor: f32) -> Option<f32> {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        Some(scale_factor)
    } else {
        None
    }
}

/// Snap a logical pixel coordinate to the nearest device pixel boundary.
///
/// This is intentionally policy-only: callers decide when to apply snapping.
pub fn snap_px_round(px: Px, scale_factor: f32) -> Px {
    let Some(sf) = normalize_scale_factor(scale_factor) else {
        return px;
    };
    Px((px.0 * sf).round() / sf)
}

/// Snap a point to device pixel boundaries (rounding each axis).
pub fn snap_point_round(point: Point, scale_factor: f32) -> Point {
    Point::new(
        snap_px_round(point.x, scale_factor),
        snap_px_round(point.y, scale_factor),
    )
}

/// Snap a rect so its *edges* land on device pixel boundaries.
///
/// This snaps left/top/right/bottom independently and derives the size from the snapped edges.
pub fn snap_rect_edges_round(rect: Rect, scale_factor: f32) -> Rect {
    let Some(sf) = normalize_scale_factor(scale_factor) else {
        return rect;
    };

    let left = rect.origin.x.0;
    let top = rect.origin.y.0;
    let right = left + rect.size.width.0;
    let bottom = top + rect.size.height.0;

    let left = (left * sf).round() / sf;
    let top = (top * sf).round() / sf;
    let right = (right * sf).round() / sf;
    let bottom = (bottom * sf).round() / sf;

    Rect::new(
        Point::new(Px(left), Px(top)),
        Size::new(Px((right - left).max(0.0)), Px((bottom - top).max(0.0))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_snapped(value: f32, scale_factor: f32) {
        let device = value * scale_factor;
        let nearest = device.round();
        assert!(
            (device - nearest).abs() < 1e-4,
            "expected {value} @ {scale_factor}x to land on a device pixel boundary: got {device}"
        );
    }

    #[test]
    fn snap_px_round_lands_on_device_pixel_boundaries() {
        let px = Px(10.1);
        let sf = 1.25;
        let snapped = snap_px_round(px, sf);
        assert_snapped(snapped.0, sf);
    }

    #[test]
    fn snap_rect_edges_round_snaps_all_edges() {
        let rect = Rect::new(Point::new(Px(10.1), Px(20.2)), Size::new(Px(12.3), Px(4.4)));
        let sf = 1.25;
        let snapped = snap_rect_edges_round(rect, sf);

        assert_snapped(snapped.origin.x.0, sf);
        assert_snapped(snapped.origin.y.0, sf);
        assert_snapped(snapped.origin.x.0 + snapped.size.width.0, sf);
        assert_snapped(snapped.origin.y.0 + snapped.size.height.0, sf);
    }
}
