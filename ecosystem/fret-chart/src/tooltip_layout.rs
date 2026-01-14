use fret_core::{Point, Px, Rect, Size};

pub(crate) fn split_tooltip_text_for_columns(text: &str) -> Option<(&str, &str)> {
    let (left, right) = text.split_once(": ")?;
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some((left, right))
}

pub(crate) fn place_tooltip_rect(
    bounds: Rect,
    anchor: Point,
    size: Size,
    offset_px: f32,
    avoid: Option<Rect>,
) -> Rect {
    fn rect_intersects(a: Rect, b: Rect) -> bool {
        let ax0 = a.origin.x.0;
        let ay0 = a.origin.y.0;
        let ax1 = ax0 + a.size.width.0;
        let ay1 = ay0 + a.size.height.0;

        let bx0 = b.origin.x.0;
        let by0 = b.origin.y.0;
        let bx1 = bx0 + b.size.width.0;
        let by1 = by0 + b.size.height.0;

        ax0 < bx1 && ax1 > bx0 && ay0 < by1 && ay1 > by0
    }

    fn rect_fits_in_bounds(rect: Rect, bounds: Rect) -> bool {
        let x0 = bounds.origin.x.0;
        let y0 = bounds.origin.y.0;
        let x1 = x0 + bounds.size.width.0;
        let y1 = y0 + bounds.size.height.0;

        let rx0 = rect.origin.x.0;
        let ry0 = rect.origin.y.0;
        let rx1 = rx0 + rect.size.width.0;
        let ry1 = ry0 + rect.size.height.0;

        rx0 >= x0 && ry0 >= y0 && rx1 <= x1 && ry1 <= y1
    }

    fn clamp_panel_origin(bounds: Rect, size: Size, origin: Point) -> Point {
        let x0 = bounds.origin.x.0;
        let y0 = bounds.origin.y.0;
        let x1 = x0 + bounds.size.width.0;
        let y1 = y0 + bounds.size.height.0;

        let w = size.width.0.max(1.0);
        let h = size.height.0.max(1.0);

        let x = if w < bounds.size.width.0 {
            origin.x.0.clamp(x0, x1 - w)
        } else {
            x0
        };
        let y = if h < bounds.size.height.0 {
            origin.y.0.clamp(y0, y1 - h)
        } else {
            y0
        };

        Point::new(Px(x), Px(y))
    }

    #[derive(Clone, Copy)]
    enum Side {
        Left,
        Right,
    }

    #[derive(Clone, Copy)]
    enum Vertical {
        Top,
        Bottom,
    }

    let w = size.width.0.max(1.0);
    let h = size.height.0.max(1.0);

    let x0 = bounds.origin.x.0;
    let y0 = bounds.origin.y.0;
    let x1 = x0 + bounds.size.width.0;
    let y1 = y0 + bounds.size.height.0;

    let right_space = x1 - (anchor.x.0 + offset_px);
    let left_space = (anchor.x.0 - offset_px) - x0;
    let top_space = (anchor.y.0 - offset_px) - y0;
    let bottom_space = y1 - (anchor.y.0 + offset_px);

    let can_right = right_space >= w;
    let can_left = left_space >= w;
    let can_top = top_space >= h;
    let can_bottom = bottom_space >= h;

    let prefer_right = right_space >= left_space;
    let prefer_top = top_space >= bottom_space;

    let primary_side = if can_right && !can_left {
        Side::Right
    } else if can_left && !can_right {
        Side::Left
    } else if prefer_right {
        Side::Right
    } else {
        Side::Left
    };

    let primary_vertical = if can_top && !can_bottom {
        Vertical::Top
    } else if can_bottom && !can_top {
        Vertical::Bottom
    } else if prefer_top {
        Vertical::Top
    } else {
        Vertical::Bottom
    };

    let other_side = match primary_side {
        Side::Left => Side::Right,
        Side::Right => Side::Left,
    };
    let other_vertical = match primary_vertical {
        Vertical::Top => Vertical::Bottom,
        Vertical::Bottom => Vertical::Top,
    };

    let candidates = [
        (primary_side, primary_vertical),
        (primary_side, other_vertical),
        (other_side, primary_vertical),
        (other_side, other_vertical),
    ];

    for (side, vertical) in candidates {
        let x = match side {
            Side::Right => anchor.x.0 + offset_px,
            Side::Left => anchor.x.0 - w - offset_px,
        };
        let y = match vertical {
            Vertical::Top => anchor.y.0 - h - offset_px,
            Vertical::Bottom => anchor.y.0 + offset_px,
        };

        let rect = Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)));
        if !rect_fits_in_bounds(rect, bounds) {
            continue;
        }
        if let Some(avoid) = avoid
            && rect_intersects(rect, avoid)
        {
            continue;
        }
        return rect;
    }

    let fallback_origin = {
        let x = match primary_side {
            Side::Right => anchor.x.0 + offset_px,
            Side::Left => anchor.x.0 - w - offset_px,
        };
        let y = match primary_vertical {
            Vertical::Top => anchor.y.0 - h - offset_px,
            Vertical::Bottom => anchor.y.0 + offset_px,
        };
        clamp_panel_origin(bounds, Size::new(Px(w), Px(h)), Point::new(Px(x), Px(y)))
    };

    Rect::new(fallback_origin, Size::new(Px(w), Px(h)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltip_column_splitter_requires_non_empty_columns() {
        assert_eq!(split_tooltip_text_for_columns("x: 1"), Some(("x", "1")));
        assert_eq!(split_tooltip_text_for_columns("x: "), None);
        assert_eq!(split_tooltip_text_for_columns(": 1"), None);
        assert_eq!(split_tooltip_text_for_columns("no delimiter"), None);
    }

    #[test]
    fn tooltip_placement_prefers_other_quadrant_when_overlapping_avoid_rect() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Point::new(Px(50.0), Px(50.0));
        let size = Size::new(Px(20.0), Px(20.0));
        let offset = 10.0;

        let avoid = Rect::new(
            Point::new(Px(55.0), Px(15.0)),
            Size::new(Px(30.0), Px(30.0)),
        );
        let placed = place_tooltip_rect(bounds, anchor, size, offset, Some(avoid));

        // Default (right-top) would overlap avoid; we expect right-bottom.
        assert_eq!(placed.origin, Point::new(Px(60.0), Px(60.0)));
    }

    #[test]
    fn tooltip_placement_flips_to_left_when_right_overflows() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Point::new(Px(95.0), Px(50.0));
        let size = Size::new(Px(20.0), Px(20.0));
        let offset = 10.0;

        let placed = place_tooltip_rect(bounds, anchor, size, offset, None);
        assert_eq!(placed.origin.x, Px(65.0));
    }

    #[test]
    fn tooltip_placement_clamps_when_panel_is_wider_than_bounds() {
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Point::new(Px(50.0), Px(50.0));
        let size = Size::new(Px(200.0), Px(20.0));
        let offset = 10.0;

        let placed = place_tooltip_rect(bounds, anchor, size, offset, None);
        assert_eq!(placed.origin.x, Px(0.0));
    }
}
