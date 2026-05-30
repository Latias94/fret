use fret_core::{Point, Px, Rect};

pub(super) fn rect_max_point(rect: Rect) -> Point {
    Point::new(
        Px(rect.origin.x.0 + rect.size.width.0),
        Px(rect.origin.y.0 + rect.size.height.0),
    )
}
