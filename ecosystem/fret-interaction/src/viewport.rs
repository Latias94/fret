//! 2D viewport transform helpers (pan/zoom mapping).
//!
//! The intent is to share a single, explicit mapping convention across multiple subsystems.
//! This avoids subtle "hand feel" drift when different layers re-implement the same math.

use fret_core::{Point, Px, Rect, Size};

/// A minimal 2D pan/zoom mapping.
///
/// Mapping convention:
/// - world -> screen: `screen = world * zoom + pan`
/// - screen -> world: `world = (screen - pan) / zoom`
///
/// `pan` is expressed in logical screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport2D {
    pub pan: Point,
    pub zoom: f32,
}

impl Viewport2D {
    pub fn new(pan: Point, zoom: f32) -> Self {
        let zoom = if zoom.is_finite() && zoom > 0.0 {
            zoom
        } else {
            1.0
        };
        Self { pan, zoom }
    }

    pub fn world_to_screen(self, p: Point) -> Point {
        Point::new(
            Px(p.x.0 * self.zoom + self.pan.x.0),
            Px(p.y.0 * self.zoom + self.pan.y.0),
        )
    }

    pub fn screen_to_world(self, p: Point) -> Point {
        Point::new(
            Px((p.x.0 - self.pan.x.0) / self.zoom),
            Px((p.y.0 - self.pan.y.0) / self.zoom),
        )
    }

    pub fn world_rect_to_screen(self, r: Rect) -> Rect {
        Rect::new(
            self.world_to_screen(r.origin),
            Size::new(
                Px(r.size.width.0 * self.zoom),
                Px(r.size.height.0 * self.zoom),
            ),
        )
    }

    pub fn screen_rect_to_world(self, r: Rect) -> Rect {
        Rect::new(
            self.screen_to_world(r.origin),
            Size::new(
                Px(r.size.width.0 / self.zoom),
                Px(r.size.height.0 / self.zoom),
            ),
        )
    }
}

impl Default for Viewport2D {
    fn default() -> Self {
        Self::new(Point::default(), 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_round_trips_points() {
        let v = Viewport2D::new(Point::new(Px(10.0), Px(-20.0)), 2.0);
        let p = Point::new(Px(3.25), Px(9.5));
        let screen = v.world_to_screen(p);
        let world = v.screen_to_world(screen);
        assert!((world.x.0 - p.x.0).abs() <= 1.0e-6);
        assert!((world.y.0 - p.y.0).abs() <= 1.0e-6);
    }

    #[test]
    fn viewport_sanitizes_zoom() {
        let v = Viewport2D::new(Point::default(), 0.0);
        assert_eq!(v.zoom, 1.0);
        let v = Viewport2D::new(Point::default(), f32::NAN);
        assert_eq!(v.zoom, 1.0);
    }
}
