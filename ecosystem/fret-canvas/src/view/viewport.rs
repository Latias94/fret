use fret_core::{Point, Rect, Transform2D};

use crate::scale::{canvas_units_from_screen_px, effective_scale_factor};

use super::{PanZoom2D, visible_canvas_rect};

/// A lightweight 2D canvas viewport mapping helper.
///
/// This is a convenience wrapper around:
/// - a widget's layout `bounds` (screen/window space), and
/// - a canvas view transform (`PanZoom2D`) used for an "infinite canvas" surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CanvasViewport2D {
    pub bounds: Rect,
    pub view: PanZoom2D,
}

impl CanvasViewport2D {
    pub fn new(bounds: Rect, view: PanZoom2D) -> Self {
        Self { bounds, view }
    }

    pub fn zoom(&self) -> f32 {
        self.view.zoom
    }

    pub fn render_transform(&self) -> Option<Transform2D> {
        self.view.render_transform(self.bounds)
    }

    pub fn screen_to_canvas(&self, screen: Point) -> Point {
        self.view.screen_to_canvas(self.bounds, screen)
    }

    pub fn canvas_to_screen(&self, canvas: Point) -> Point {
        self.view.canvas_to_screen(self.bounds, canvas)
    }

    /// Returns the visible canvas-space rectangle for this viewport.
    pub fn visible_canvas_rect(&self) -> Rect {
        visible_canvas_rect(self.bounds, self.view)
    }

    /// Converts a screen-space logical pixel length into canvas-space units under this viewport.
    pub fn canvas_units_from_screen_px(&self, screen_px: f32) -> f32 {
        canvas_units_from_screen_px(screen_px, self.view.zoom)
    }

    /// Returns the effective preparation scale factor for GPU-backed resources under this viewport.
    pub fn effective_scale_factor(&self, window_scale_factor: f32) -> f32 {
        effective_scale_factor(window_scale_factor, self.view.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Px;

    #[test]
    fn canvas_to_screen_round_trips_with_screen_to_canvas() {
        let bounds = Rect::new(
            Point::new(Px(10.0), Px(20.0)),
            fret_core::Size::new(Px(800.0), Px(600.0)),
        );
        let viewport = CanvasViewport2D::new(
            bounds,
            PanZoom2D {
                pan: Point::new(Px(-3.0), Px(5.0)),
                zoom: 2.0,
            },
        );

        let p0 = Point::new(Px(200.0), Px(150.0));
        let canvas = viewport.screen_to_canvas(p0);
        let p1 = viewport.canvas_to_screen(canvas);
        assert!((p0.x.0 - p1.x.0).abs() <= 1.0e-6);
        assert!((p0.y.0 - p1.y.0).abs() <= 1.0e-6);
    }
}
