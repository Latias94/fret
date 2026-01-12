use fret_core::Px;

/// Converts a screen-space length in logical pixels into canvas-space units under a uniform `zoom`.
#[inline]
pub fn canvas_units_from_screen_px(screen_px: f32, zoom: f32) -> f32 {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    screen_px / zoom
}

/// Returns a canvas-space stroke width that remains approximately constant in screen pixels.
#[inline]
pub fn constant_pixel_stroke_width(stroke_width_px: Px, zoom: f32) -> Px {
    Px(canvas_units_from_screen_px(stroke_width_px.0, zoom))
}

/// Returns the effective scale factor for preparing GPU-backed resources under a canvas zoom.
///
/// This is commonly used for path/text preparation constraints so the prepared resolution
/// tracks both window DPI (`scale_factor`) and view zoom.
#[inline]
pub fn effective_scale_factor(scale_factor: f32, zoom: f32) -> f32 {
    let zoom = if zoom.is_finite() && zoom > 0.0 {
        zoom
    } else {
        1.0
    };
    scale_factor * zoom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_pixel_stroke_scales_inversely() {
        let w = constant_pixel_stroke_width(Px(2.0), 4.0);
        assert!((w.0 - 0.5).abs() <= 1.0e-9);
    }
}
