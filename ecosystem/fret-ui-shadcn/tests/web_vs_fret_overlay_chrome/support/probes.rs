use super::*;

pub(crate) fn leftish_text_probe_point(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + 40.0),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}
