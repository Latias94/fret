use fret_core::{Point, Px, Rect};
pub fn rect_center(bounds: Rect) -> Point {
    Point::new(
        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
    )
}

pub fn ripple_max_radius(bounds: Rect, origin: Point) -> Px {
    let w = bounds.size.width.0.max(0.0);
    let h = bounds.size.height.0.max(0.0);
    let ox = (origin.x.0 - bounds.origin.x.0).clamp(0.0, w) + bounds.origin.x.0;
    let oy = (origin.y.0 - bounds.origin.y.0).clamp(0.0, h) + bounds.origin.y.0;
    let left = bounds.origin.x.0;
    let top = bounds.origin.y.0;
    let right = left + w;
    let bottom = top + h;
    let corners = [(left, top), (right, top), (left, bottom), (right, bottom)];
    let mut max: f32 = 0.0;
    for (cx, cy) in corners {
        let dx = cx - ox;
        let dy = cy - oy;
        max = max.max((dx * dx + dy * dy).sqrt());
    }
    Px(max)
}
