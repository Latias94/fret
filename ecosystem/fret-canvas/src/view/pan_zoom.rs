use crate::scale::canvas_units_from_screen_px;
use fret_core::{Point, Px, Rect, Transform2D};

/// Default zoom base used by canvas wheel zoom curves (ADR 0159).
pub const DEFAULT_WHEEL_ZOOM_BASE: f32 = 1.18;

/// Default wheel delta step used by canvas wheel zoom curves (ADR 0159).
pub const DEFAULT_WHEEL_ZOOM_STEP: f32 = 120.0;

/// Computes the wheel zoom factor for a given vertical wheel delta (screen px).
///
/// This matches ADR 0159's deterministic exponential zoom curve:
/// `base.powf((-delta_y / step) * speed)`.
pub fn wheel_zoom_factor(delta_y_screen_px: f32, base: f32, step: f32, speed: f32) -> Option<f32> {
    if !delta_y_screen_px.is_finite() || delta_y_screen_px.abs() <= 1.0e-9 {
        return None;
    }
    if !base.is_finite() || base <= 0.0 {
        return None;
    }
    if !step.is_finite() || step.abs() <= 1.0e-9 {
        return None;
    }
    let speed = if speed.is_finite() { speed } else { 1.0 };
    Some(base.powf((-delta_y_screen_px / step) * speed))
}

/// A simple 2D pan/zoom view model suitable for "infinite canvas" widgets.
///
/// Coordinate conventions match the common retained-canvas pattern:
/// - Content is authored in **canvas space**.
/// - A view transform maps canvas space to window space via:
///   `T(bounds.origin) * S(zoom) * T(pan)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanZoom2D {
    /// Canvas-space translation applied after scaling.
    pub pan: Point,
    /// Uniform scale factor.
    pub zoom: f32,
}

impl Default for PanZoom2D {
    fn default() -> Self {
        Self {
            pan: Point::new(Px(0.0), Px(0.0)),
            zoom: 1.0,
        }
    }
}

impl PanZoom2D {
    /// Returns `true` when `zoom` is finite and positive.
    pub fn zoom_is_valid(&self) -> bool {
        self.zoom.is_finite() && self.zoom > 0.0
    }

    /// Returns a sanitized zoom value, falling back to `fallback` when invalid.
    pub fn sanitize_zoom(zoom: f32, fallback: f32) -> f32 {
        if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            fallback
        }
    }

    /// Computes the render transform for a widget with the given layout `bounds`.
    ///
    /// Returns `None` when the transform would be non-invertible.
    pub fn render_transform(&self, bounds: Rect) -> Option<Transform2D> {
        if !self.zoom_is_valid() {
            return None;
        }

        let t = Transform2D::translation(bounds.origin)
            .compose(Transform2D::scale_uniform(self.zoom))
            .compose(Transform2D::translation(self.pan));
        t.inverse().is_some().then_some(t)
    }

    /// Converts a screen-space delta in logical pixels into a canvas-space delta under this view.
    #[inline]
    pub fn canvas_delta_from_screen_delta(&self, dx_screen_px: f32, dy_screen_px: f32) -> Point {
        let zoom = Self::sanitize_zoom(self.zoom, 1.0);
        Point::new(
            Px(canvas_units_from_screen_px(dx_screen_px, zoom)),
            Px(canvas_units_from_screen_px(dy_screen_px, zoom)),
        )
    }

    /// Applies a screen-space delta (logical pixels) as a canvas-space pan delta.
    ///
    /// Positive deltas move the content in the same direction as pointer dragging.
    #[inline]
    pub fn pan_by_screen_delta(&mut self, dx_screen_px: f32, dy_screen_px: f32) {
        let delta = self.canvas_delta_from_screen_delta(dx_screen_px, dy_screen_px);
        self.pan.x = Px(self.pan.x.0 + delta.x.0);
        self.pan.y = Px(self.pan.y.0 + delta.y.0);
    }

    /// Maps a window-space point into canvas space.
    pub fn screen_to_canvas(&self, bounds: Rect, screen: Point) -> Point {
        let zoom = Self::sanitize_zoom(self.zoom, 1.0);
        let x = (screen.x.0 - bounds.origin.x.0) / zoom - self.pan.x.0;
        let y = (screen.y.0 - bounds.origin.y.0) / zoom - self.pan.y.0;
        Point::new(Px(x), Px(y))
    }

    /// Maps a canvas-space point into window space.
    pub fn canvas_to_screen(&self, bounds: Rect, canvas: Point) -> Point {
        let zoom = Self::sanitize_zoom(self.zoom, 1.0);
        let x = bounds.origin.x.0 + zoom * (canvas.x.0 + self.pan.x.0);
        let y = bounds.origin.y.0 + zoom * (canvas.y.0 + self.pan.y.0);
        Point::new(Px(x), Px(y))
    }

    /// Adjusts `pan` so that zooming keeps the canvas point under `center_screen` stable.
    pub fn zoom_about_screen_point(&mut self, bounds: Rect, center_screen: Point, new_zoom: f32) {
        let zoom = Self::sanitize_zoom(self.zoom, 1.0);
        let new_zoom = Self::sanitize_zoom(new_zoom, zoom);
        if (new_zoom - zoom).abs() <= 1.0e-9 {
            return;
        }

        let g0 = self.screen_to_canvas(bounds, center_screen);
        let new_pan_x = (center_screen.x.0 - bounds.origin.x.0) / new_zoom - g0.x.0;
        let new_pan_y = (center_screen.y.0 - bounds.origin.y.0) / new_zoom - g0.y.0;
        self.pan = Point::new(Px(new_pan_x), Px(new_pan_y));
        self.zoom = new_zoom;
    }

    /// Adjusts `pan` so that zooming keeps the given canvas-space point stable in screen space.
    ///
    /// This is equivalent to "zoom about cursor" in widgets that report pointer positions in
    /// canvas space (e.g. when the widget uses a pan/zoom `render_transform`).
    ///
    /// The stability guarantee is relative to the widget's origin; callers can add any `bounds`
    /// origin externally if they need absolute window-space coordinates.
    pub fn zoom_about_canvas_point(&mut self, center_canvas: Point, new_zoom: f32) {
        let zoom = Self::sanitize_zoom(self.zoom, 1.0);
        let new_zoom = Self::sanitize_zoom(new_zoom, zoom);
        if (new_zoom - zoom).abs() <= 1.0e-9 {
            return;
        }

        let scale = zoom / new_zoom;
        let new_pan_x = (center_canvas.x.0 + self.pan.x.0) * scale - center_canvas.x.0;
        let new_pan_y = (center_canvas.y.0 + self.pan.y.0) * scale - center_canvas.y.0;

        self.pan = Point::new(Px(new_pan_x), Px(new_pan_y));
        self.zoom = new_zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_zoom_factor_matches_default_step() {
        let got = wheel_zoom_factor(
            -120.0,
            DEFAULT_WHEEL_ZOOM_BASE,
            DEFAULT_WHEEL_ZOOM_STEP,
            1.0,
        )
        .unwrap();
        assert!((got - 1.18).abs() <= 1.0e-6);
        let got_out =
            wheel_zoom_factor(120.0, DEFAULT_WHEEL_ZOOM_BASE, DEFAULT_WHEEL_ZOOM_STEP, 1.0)
                .unwrap();
        assert!((got_out - (1.0 / 1.18)).abs() <= 1.0e-6);
    }

    #[test]
    fn zoom_about_keeps_center_stable() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let mut view = PanZoom2D {
            pan: Point::new(Px(-3.0), Px(5.0)),
            zoom: 2.0,
        };

        let center = Point::new(Px(200.0), Px(150.0));
        let before = view.screen_to_canvas(bounds, center);
        view.zoom_about_screen_point(bounds, center, 3.0);
        let after = view.screen_to_canvas(bounds, center);

        assert!((before.x.0 - after.x.0).abs() <= 1.0e-6);
        assert!((before.y.0 - after.y.0).abs() <= 1.0e-6);
    }

    #[test]
    fn zoom_about_canvas_point_keeps_screen_relative_position_stable() {
        let mut view = PanZoom2D {
            pan: Point::new(Px(-3.0), Px(5.0)),
            zoom: 2.0,
        };

        let center_canvas = Point::new(Px(42.0), Px(-17.0));
        let before_x = (center_canvas.x.0 + view.pan.x.0) * view.zoom;
        let before_y = (center_canvas.y.0 + view.pan.y.0) * view.zoom;

        view.zoom_about_canvas_point(center_canvas, 3.0);

        let after_x = (center_canvas.x.0 + view.pan.x.0) * view.zoom;
        let after_y = (center_canvas.y.0 + view.pan.y.0) * view.zoom;

        assert!((before_x - after_x).abs() <= 1.0e-6);
        assert!((before_y - after_y).abs() <= 1.0e-6);
    }

    #[test]
    fn pan_by_screen_delta_scales_by_zoom() {
        let mut view = PanZoom2D {
            pan: Point::new(Px(0.0), Px(0.0)),
            zoom: 2.0,
        };

        view.pan_by_screen_delta(10.0, -6.0);
        assert!((view.pan.x.0 - 5.0).abs() <= 1.0e-9);
        assert!((view.pan.y.0 - (-3.0)).abs() <= 1.0e-9);
    }
}
