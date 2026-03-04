use fret_core::geometry::Point;
use fret_core::{Px, Rect};

pub fn tab_close_hit_test(bounds: Rect, pointer: Point, close_size: Px, padding_right: Px) -> bool {
    let close_size = close_size.0.max(0.0);
    let padding_right = padding_right.0.max(0.0);
    let close_x0 = bounds.origin.x.0 + (bounds.size.width.0 - padding_right - close_size);
    let close_y0 = bounds.origin.y.0 + (bounds.size.height.0 - close_size) * 0.5;

    pointer.x.0 >= close_x0
        && pointer.x.0 <= close_x0 + close_size
        && pointer.y.0 >= close_y0
        && pointer.y.0 <= close_y0 + close_size
}

/// Returns true when `end` remains within an axis-aligned slop box around `start`.
///
/// This is intended for click arbitration: treat small pointer jitter as still "clicked" even if
/// the pointer-up position is outside the original hit target.
pub fn pointer_move_within_slop(start: Point, end: Point, slop: Px) -> bool {
    let slop = slop.0.max(0.0);
    let dx = (end.x.0 - start.x.0).abs();
    let dy = (end.y.0 - start.y.0).abs();
    dx.max(dy) <= slop
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::Size;

    #[test]
    fn tab_close_hit_test_matches_expected_rect() {
        let bounds = Rect::new(
            Point::new(Px(20.0), Px(10.0)),
            Size::new(Px(100.0), Px(24.0)),
        );
        let close_size = Px(18.0);
        let padding_right = Px(6.0);

        assert!(
            tab_close_hit_test(
                bounds,
                Point::new(Px(20.0 + 80.0), Px(10.0 + 10.0)),
                close_size,
                padding_right
            ),
            "pointer inside expected close rect should hit"
        );
        assert!(
            !tab_close_hit_test(
                bounds,
                Point::new(Px(20.0 + 70.0), Px(10.0 + 10.0)),
                close_size,
                padding_right
            ),
            "pointer left of expected close rect should miss"
        );
    }

    #[test]
    fn pointer_move_within_slop_accepts_small_jitter() {
        let start = Point::new(Px(100.0), Px(200.0));
        assert!(pointer_move_within_slop(
            start,
            Point::new(Px(103.0), Px(199.0)),
            Px(4.0)
        ));
        assert!(!pointer_move_within_slop(
            start,
            Point::new(Px(105.0), Px(200.0)),
            Px(4.0)
        ));
    }
}
