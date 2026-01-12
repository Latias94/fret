use fret_core::{Point, Px, Rect, Transform2D};

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
