use fret_core::{Point, Px, Rect, Size};

pub(super) fn node_rect(position: crate::core::CanvasPoint, size: Size) -> Rect {
    Rect::new(Point::new(Px(position.x), Px(position.y)), size)
}
