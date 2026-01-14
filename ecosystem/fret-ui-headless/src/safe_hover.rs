//! Deterministic "safe hover" corridor helpers for nested menus.
//!
//! This is inspired by Floating UI's `safePolygon` idea, but intentionally kept simple and
//! deterministic: we provide a geometry-only corridor test (no velocity/intention heuristics).
//!
//! Typical use: keep a submenu open while the pointer moves diagonally from the submenu trigger
//! into the submenu panel.

use fret_core::{Point, Px, Rect, Size};

fn rect_expand(rect: Rect, px: Px) -> Rect {
    Rect::new(
        Point::new(rect.origin.x - px, rect.origin.y - px),
        Size::new(rect.size.width + px * 2.0, rect.size.height + px * 2.0),
    )
}

fn is_point_in_polygon(point: Point, polygon: &[Point]) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let x = point.x.0;
    let y = point.y.0;
    let mut inside = false;
    let mut j = polygon.len() - 1;
    for i in 0..polygon.len() {
        let xi = polygon[i].x.0;
        let yi = polygon[i].y.0;
        let xj = polygon[j].x.0;
        let yj = polygon[j].y.0;

        let intersect = (yi >= y) != (yj >= y) && x <= ((xj - xi) * (y - yi)) / (yj - yi) + xi;
        if intersect {
            inside = !inside;
        }
        j = i;
    }

    inside
}

/// Returns `true` if `point` lies within the "safe hover" corridor between `reference` and `floating`.
///
/// - `reference`: submenu trigger bounds
/// - `floating`: submenu panel bounds
/// - `buffer`: expands both rectangles and the corridor to reduce accidental closes
///
/// This implementation supports common overlay topologies (left/right/above/below) and returns
/// `false` for overlapping geometries, which keeps the behavior predictable.
pub fn safe_hover_contains(point: Point, reference: Rect, floating: Rect, buffer: Px) -> bool {
    let reference = rect_expand(reference, buffer);
    let floating = rect_expand(floating, buffer);
    if reference.contains(point) || floating.contains(point) {
        return true;
    }

    let ref_left = reference.origin.x.0;
    let ref_right = ref_left + reference.size.width.0;
    let ref_top = reference.origin.y.0;
    let ref_bottom = ref_top + reference.size.height.0;

    let float_left = floating.origin.x.0;
    let float_right = float_left + floating.size.width.0;
    let float_top = floating.origin.y.0;
    let float_bottom = float_top + floating.size.height.0;

    if float_left >= ref_right {
        let poly = [
            Point::new(Px(ref_right), Px(ref_top)),
            Point::new(Px(float_left), Px(float_top)),
            Point::new(Px(float_left), Px(float_bottom)),
            Point::new(Px(ref_right), Px(ref_bottom)),
        ];
        return is_point_in_polygon(point, &poly);
    }

    if float_right <= ref_left {
        let poly = [
            Point::new(Px(float_right), Px(float_top)),
            Point::new(Px(ref_left), Px(ref_top)),
            Point::new(Px(ref_left), Px(ref_bottom)),
            Point::new(Px(float_right), Px(float_bottom)),
        ];
        return is_point_in_polygon(point, &poly);
    }

    if float_bottom <= ref_top {
        let poly = [
            Point::new(Px(ref_left), Px(ref_top)),
            Point::new(Px(ref_right), Px(ref_top)),
            Point::new(Px(float_right), Px(float_bottom)),
            Point::new(Px(float_left), Px(float_bottom)),
        ];
        return is_point_in_polygon(point, &poly);
    }

    if float_top >= ref_bottom {
        let poly = [
            Point::new(Px(float_left), Px(float_top)),
            Point::new(Px(float_right), Px(float_top)),
            Point::new(Px(ref_right), Px(ref_bottom)),
            Point::new(Px(ref_left), Px(ref_bottom)),
        ];
        return is_point_in_polygon(point, &poly);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_point_inside_reference_or_floating() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

        assert!(safe_hover_contains(
            Point::new(Px(5.0), Px(5.0)),
            reference,
            floating,
            Px(0.0)
        ));
        assert!(safe_hover_contains(
            Point::new(Px(25.0), Px(5.0)),
            reference,
            floating,
            Px(0.0)
        ));
    }

    #[test]
    fn contains_point_in_trapezoid_corridor() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(2.0)), Size::new(Px(10.0), Px(10.0)));

        // Roughly diagonal from the right edge of reference to the left edge of floating.
        assert!(safe_hover_contains(
            Point::new(Px(12.0), Px(5.0)),
            reference,
            floating,
            Px(0.0)
        ));
    }

    #[test]
    fn rejects_point_outside_corridor() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(20.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

        assert!(!safe_hover_contains(
            Point::new(Px(12.0), Px(30.0)),
            reference,
            floating,
            Px(0.0)
        ));
    }

    #[test]
    fn contains_point_in_trapezoid_corridor_above() {
        let reference = Rect::new(Point::new(Px(0.0), Px(20.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(2.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));

        // Roughly diagonal from the top edge of reference to the bottom edge of floating.
        assert!(safe_hover_contains(
            Point::new(Px(6.0), Px(15.0)),
            reference,
            floating,
            Px(0.0)
        ));
    }

    #[test]
    fn contains_point_in_trapezoid_corridor_below() {
        let reference = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(10.0), Px(10.0)));
        let floating = Rect::new(Point::new(Px(2.0), Px(20.0)), Size::new(Px(10.0), Px(10.0)));

        // Roughly diagonal from the bottom edge of reference to the top edge of floating.
        assert!(safe_hover_contains(
            Point::new(Px(6.0), Px(15.0)),
            reference,
            floating,
            Px(0.0)
        ));
    }
}
