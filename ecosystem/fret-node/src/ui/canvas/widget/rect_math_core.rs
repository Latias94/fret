use fret_core::{Point, Px, Rect, Size};

pub(super) fn rect_from_points(a: Point, b: Point) -> Rect {
    let min_x = a.x.0.min(b.x.0);
    let min_y = a.y.0.min(b.y.0);
    let max_x = a.x.0.max(b.x.0);
    let max_y = a.y.0.max(b.y.0);
    rect_from_extents(min_x, min_y, max_x, max_y)
}

pub(super) fn rect_union(a: Rect, b: Rect) -> Rect {
    let (ax0, ay0, ax1, ay1) = rect_extents(a);
    let (bx0, by0, bx1, by1) = rect_extents(b);
    rect_from_extents(ax0.min(bx0), ay0.min(by0), ax1.max(bx1), ay1.max(by1))
}

pub(super) fn rects_intersect(a: Rect, b: Rect) -> bool {
    let (ax0, ay0, ax1, ay1) = rect_extents(a);
    let (bx0, by0, bx1, by1) = rect_extents(b);
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

fn rect_extents(rect: Rect) -> (f32, f32, f32, f32) {
    (
        rect.origin.x.0,
        rect.origin.y.0,
        rect.origin.x.0 + rect.size.width.0,
        rect.origin.y.0 + rect.size.height.0,
    )
}

fn rect_from_extents(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Rect {
    Rect::new(
        Point::new(Px(min_x), Px(min_y)),
        Size::new(Px(max_x - min_x), Px(max_y - min_y)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_from_points_orders_corners() {
        let rect = rect_from_points(
            Point::new(Px(30.0), Px(10.0)),
            Point::new(Px(5.0), Px(25.0)),
        );
        assert_eq!(
            rect,
            Rect::new(Point::new(Px(5.0), Px(10.0)), Size::new(Px(25.0), Px(15.0)))
        );
    }

    #[test]
    fn rects_intersect_counts_touching_edges_as_intersection() {
        let a = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let b = Rect::new(Point::new(Px(10.0), Px(5.0)), Size::new(Px(4.0), Px(6.0)));
        assert!(rects_intersect(a, b));
    }
}
