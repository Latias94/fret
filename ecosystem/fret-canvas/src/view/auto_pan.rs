use fret_core::{Point, Px, Rect};

use super::PanZoom2D;

/// Auto-pan tuning for infinite-canvas interactions (node drag, connect drag, etc.).
///
/// Values are specified in **screen-space logical pixels** so behavior remains stable under zoom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoPanTuning {
    /// Margin near the viewport edges that triggers auto-pan (screen px).
    pub margin_screen_px: f32,
    /// Max auto-pan speed when the pointer is at the edge (screen px / second).
    pub speed_screen_px_per_s: f32,
}

impl AutoPanTuning {
    pub fn is_valid(&self) -> bool {
        self.margin_screen_px.is_finite()
            && self.margin_screen_px > 0.0
            && self.speed_screen_px_per_s.is_finite()
            && self.speed_screen_px_per_s > 0.0
    }
}

/// Computes a canvas-space pan delta for a single auto-pan "tick".
///
/// - `bounds` is the viewport bounds in window space.
/// - `view` is the current pan/zoom transform.
/// - `canvas_pos` is the current pointer position in canvas space.
/// - `tick_hz` is the tick rate (e.g. 60Hz) used by the caller's timer.
///
/// The returned delta is in canvas units and is meant to be applied to `view.pan` (or an equivalent
/// canvas-space pan vector).
pub fn auto_pan_delta_per_tick(
    bounds: Rect,
    view: PanZoom2D,
    canvas_pos: Point,
    tuning: AutoPanTuning,
    tick_hz: f32,
) -> Point {
    if !view.zoom_is_valid() {
        return Point::new(Px(0.0), Px(0.0));
    }
    if !tuning.is_valid() {
        return Point::new(Px(0.0), Px(0.0));
    }
    if !tick_hz.is_finite() || tick_hz <= 0.0 {
        return Point::new(Px(0.0), Px(0.0));
    }

    let viewport_w = bounds.size.width.0;
    let viewport_h = bounds.size.height.0;
    if !viewport_w.is_finite() || viewport_w <= 0.0 || !viewport_h.is_finite() || viewport_h <= 0.0
    {
        return Point::new(Px(0.0), Px(0.0));
    }

    let pos_screen = view.canvas_to_screen(bounds, canvas_pos);
    let pos_local_x = pos_screen.x.0 - bounds.origin.x.0;
    let pos_local_y = pos_screen.y.0 - bounds.origin.y.0;

    let dist_left = pos_local_x;
    let dist_right = viewport_w - pos_local_x;
    let dist_top = pos_local_y;
    let dist_bottom = viewport_h - pos_local_y;

    let step_screen = tuning.speed_screen_px_per_s / tick_hz;
    let step_canvas = step_screen / view.zoom;

    let margin_screen = tuning.margin_screen_px;

    let mut delta_x = 0.0;
    let mut delta_y = 0.0;

    if dist_left.is_finite() && dist_left < margin_screen {
        let factor = ((margin_screen - dist_left) / margin_screen).clamp(0.0, 1.0);
        delta_x += step_canvas * factor;
    }
    if dist_right.is_finite() && dist_right < margin_screen {
        let factor = ((margin_screen - dist_right) / margin_screen).clamp(0.0, 1.0);
        delta_x -= step_canvas * factor;
    }
    if dist_top.is_finite() && dist_top < margin_screen {
        let factor = ((margin_screen - dist_top) / margin_screen).clamp(0.0, 1.0);
        delta_y += step_canvas * factor;
    }
    if dist_bottom.is_finite() && dist_bottom < margin_screen {
        let factor = ((margin_screen - dist_bottom) / margin_screen).clamp(0.0, 1.0);
        delta_y -= step_canvas * factor;
    }

    if !delta_x.is_finite() || !delta_y.is_finite() {
        return Point::new(Px(0.0), Px(0.0));
    }

    Point::new(Px(delta_x), Px(delta_y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn bounds_at(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn auto_pan_is_zero_when_pointer_is_not_near_edges() {
        let bounds = bounds_at(0.0, 0.0, 800.0, 600.0);
        let view = PanZoom2D::default();
        let tuning = AutoPanTuning {
            margin_screen_px: 80.0,
            speed_screen_px_per_s: 120.0,
        };

        let center_screen = Point::new(Px(400.0), Px(300.0));
        let center_canvas = view.screen_to_canvas(bounds, center_screen);
        let delta = auto_pan_delta_per_tick(bounds, view, center_canvas, tuning, 60.0);
        assert_eq!(delta, Point::new(Px(0.0), Px(0.0)));
    }

    #[test]
    fn auto_pan_scales_in_screen_space_under_zoom() {
        let bounds = bounds_at(0.0, 0.0, 800.0, 600.0);
        let tuning = AutoPanTuning {
            margin_screen_px: 80.0,
            speed_screen_px_per_s: 60.0,
        };

        let view_zoom_1 = PanZoom2D::default();
        let screen_left_center = Point::new(Px(0.0), Px(300.0));
        let canvas_left_center_zoom_1 = view_zoom_1.screen_to_canvas(bounds, screen_left_center);

        let delta_zoom_1 =
            auto_pan_delta_per_tick(bounds, view_zoom_1, canvas_left_center_zoom_1, tuning, 60.0);
        assert!((delta_zoom_1.x.0 - 1.0).abs() <= 1.0e-6);
        assert!((delta_zoom_1.y.0).abs() <= 1.0e-6);

        let view_zoom_2 = PanZoom2D {
            pan: Point::new(Px(0.0), Px(0.0)),
            zoom: 2.0,
        };
        let canvas_left_center_zoom_2 = view_zoom_2.screen_to_canvas(bounds, screen_left_center);

        let delta_zoom_2 =
            auto_pan_delta_per_tick(bounds, view_zoom_2, canvas_left_center_zoom_2, tuning, 60.0);
        assert!((delta_zoom_2.x.0 - 0.5).abs() <= 1.0e-6);
        assert!((delta_zoom_2.y.0).abs() <= 1.0e-6);
    }

    #[test]
    fn auto_pan_uses_bounds_origin() {
        let bounds = bounds_at(100.0, 50.0, 800.0, 600.0);
        let tuning = AutoPanTuning {
            margin_screen_px: 80.0,
            speed_screen_px_per_s: 60.0,
        };

        let view = PanZoom2D::default();
        let screen_left_center = Point::new(Px(100.0), Px(50.0 + 300.0));
        let canvas_left_center = view.screen_to_canvas(bounds, screen_left_center);

        let delta = auto_pan_delta_per_tick(bounds, view, canvas_left_center, tuning, 60.0);
        assert!((delta.x.0 - 1.0).abs() <= 1.0e-6);
        assert!((delta.y.0).abs() <= 1.0e-6);
    }
}
