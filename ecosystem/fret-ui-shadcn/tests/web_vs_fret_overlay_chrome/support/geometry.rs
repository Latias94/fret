use super::*;

pub(crate) fn rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}

pub(crate) fn rect_area(rect: Rect) -> f32 {
    rect.size.width.0 * rect.size.height.0
}

pub(crate) fn rect_intersection_area(a: Rect, b: Rect) -> f32 {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = ax0 + a.size.width.0;
    let ay1 = ay0 + a.size.height.0;
    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = bx0 + b.size.width.0;
    let by1 = by0 + b.size.height.0;

    let x0 = ax0.max(bx0);
    let y0 = ay0.max(by0);
    let x1 = ax1.min(bx1);
    let y1 = ay1.min(by1);

    let w = (x1 - x0).max(0.0);
    let h = (y1 - y0).max(0.0);
    w * h
}

pub(crate) fn bounds_center(r: Rect) -> Point {
    Point::new(
        Px(r.origin.x.0 + r.size.width.0 * 0.5),
        Px(r.origin.y.0 + r.size.height.0 * 0.5),
    )
}

pub(crate) fn rect_contains_point_with_margin(rect: Rect, point: Point, margin_px: f32) -> bool {
    let left = rect.origin.x.0 - margin_px;
    let top = rect.origin.y.0 - margin_px;
    let right = rect.origin.x.0 + rect.size.width.0 + margin_px;
    let bottom = rect.origin.y.0 + rect.size.height.0 + margin_px;

    point.x.0 >= left && point.x.0 <= right && point.y.0 >= top && point.y.0 <= bottom
}
