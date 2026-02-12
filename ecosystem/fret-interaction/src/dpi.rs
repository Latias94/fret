//! DPI helpers shared by interaction primitives.
//!
//! These are intentionally small and policy-light: callers remain responsible for selecting the
//! appropriate scale factor and applying snapping only where it is correct for the surface.

use fret_core::{Point, Px, Size};

pub fn snap_px_to_device_pixels(scale_factor: f32, px: Px) -> Px {
    if !scale_factor.is_finite() || scale_factor <= 0.0 {
        return px;
    }
    if !px.0.is_finite() {
        return px;
    }
    Px((px.0 * scale_factor).round() / scale_factor)
}

pub fn snap_point_to_device_pixels(scale_factor: f32, p: Point) -> Point {
    Point::new(
        snap_px_to_device_pixels(scale_factor, p.x),
        snap_px_to_device_pixels(scale_factor, p.y),
    )
}

pub fn snap_size_to_device_pixels(scale_factor: f32, s: Size) -> Size {
    Size::new(
        snap_px_to_device_pixels(scale_factor, s.width),
        snap_px_to_device_pixels(scale_factor, s.height),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapping_with_invalid_scale_is_identity() {
        assert_eq!(snap_px_to_device_pixels(0.0, Px(1.2)), Px(1.2));
        assert_eq!(snap_px_to_device_pixels(f32::NAN, Px(1.2)), Px(1.2));
        assert_eq!(snap_px_to_device_pixels(-2.0, Px(1.2)), Px(1.2));
    }

    #[test]
    fn snapping_rounds_to_device_pixel_grid() {
        // 150% scale => device pixels are 2/3 logical px steps.
        let got = snap_px_to_device_pixels(1.5, Px(0.7));
        // 0.7 * 1.5 = 1.05 -> round = 1 -> 1 / 1.5 = 0.666...
        assert!((got.0 - (2.0 / 3.0)).abs() <= 1.0e-6);
    }
}
