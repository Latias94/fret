use fret_core::{Point, Px, Rect, Size};

use crate::core::CanvasPoint;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeGraphCanvasTransform {
    pub bounds_origin: Point,
    pub bounds_size: Size,
    pub pan: CanvasPoint,
    pub zoom: f32,
}

impl Default for NodeGraphCanvasTransform {
    fn default() -> Self {
        Self {
            bounds_origin: Point::new(Px(0.0), Px(0.0)),
            bounds_size: Size::new(Px(0.0), Px(0.0)),
            pan: CanvasPoint::default(),
            zoom: 1.0,
        }
    }
}

impl NodeGraphCanvasTransform {
    pub fn canvas_point_to_window(self, p: Point) -> Point {
        let z = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        Point::new(
            Px(self.bounds_origin.x.0 + (p.x.0 + self.pan.x) * z),
            Px(self.bounds_origin.y.0 + (p.y.0 + self.pan.y) * z),
        )
    }

    pub fn canvas_rect_to_window(self, r: Rect) -> Rect {
        let z = if self.zoom.is_finite() && self.zoom > 0.0 {
            self.zoom
        } else {
            1.0
        };
        let origin = Point::new(
            Px(self.bounds_origin.x.0 + (r.origin.x.0 + self.pan.x) * z),
            Px(self.bounds_origin.y.0 + (r.origin.y.0 + self.pan.y) * z),
        );
        let size = Size::new(Px(r.size.width.0 * z), Px(r.size.height.0 * z));
        Rect::new(origin, size)
    }
}
