use super::*;

pub(super) fn rect_from_points(a: Point, b: Point) -> Rect {
    let min_x = a.x.0.min(b.x.0);
    let min_y = a.y.0.min(b.y.0);
    let max_x = a.x.0.max(b.x.0);
    let max_y = a.y.0.max(b.y.0);
    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

pub(super) fn rect_union(a: Rect, b: Rect) -> Rect {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = a.origin.x.0 + a.size.width.0;
    let ay1 = a.origin.y.0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = b.origin.x.0 + b.size.width.0;
    let by1 = b.origin.y.0 + b.size.height.0;

    let min_x = ax0.min(bx0);
    let min_y = ay0.min(by0);
    let max_x = ax1.max(bx1);
    let max_y = ay1.max(by1);

    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

pub(super) fn rects_intersect(a: Rect, b: Rect) -> bool {
    let ax0 = a.origin.x.0;
    let ay0 = a.origin.y.0;
    let ax1 = a.origin.x.0 + a.size.width.0;
    let ay1 = a.origin.y.0 + a.size.height.0;

    let bx0 = b.origin.x.0;
    let by0 = b.origin.y.0;
    let bx1 = b.origin.x.0 + b.size.width.0;
    let by1 = b.origin.y.0 + b.size.height.0;

    ax0 <= bx1 && ax1 >= bx0 && ay0 <= by1 && ay1 >= by0
}

pub(super) fn inflate_rect(rect: Rect, margin: f32) -> Rect {
    if !margin.is_finite() || margin <= 0.0 {
        return rect;
    }
    Rect::new(
        Point::new(Px(rect.origin.x.0 - margin), Px(rect.origin.y.0 - margin)),
        Size::new(
            Px(rect.size.width.0 + 2.0 * margin),
            Px(rect.size.height.0 + 2.0 * margin),
        ),
    )
}

pub(super) fn edge_bounds_rect(
    route: EdgeRouteKind,
    from: Point,
    to: Point,
    zoom: f32,
    pad: f32,
) -> Rect {
    let mut min_x = from.x.0.min(to.x.0);
    let mut min_y = from.y.0.min(to.y.0);
    let mut max_x = from.x.0.max(to.x.0);
    let mut max_y = from.y.0.max(to.y.0);

    if route == EdgeRouteKind::Bezier {
        let (c1, c2) = wire_ctrl_points(from, to, zoom);
        min_x = min_x.min(c1.x.0).min(c2.x.0);
        min_y = min_y.min(c1.y.0).min(c2.y.0);
        max_x = max_x.max(c1.x.0).max(c2.x.0);
        max_y = max_y.max(c1.y.0).max(c2.y.0);
    }

    let pad = if pad.is_finite() { pad.max(0.0) } else { 0.0 };

    Rect::new(
        Point::new(Px(min_x - pad), Px(min_y - pad)),
        Size::new(
            Px((max_x - min_x) + 2.0 * pad),
            Px((max_y - min_y) + 2.0 * pad),
        ),
    )
}
