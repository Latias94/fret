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

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Rect, Size, px};

    #[test]
    fn tab_close_hit_test_matches_expected_rect() {
        let bounds = Rect::new(Point::new(px(20.0), px(10.0)), Size::new(px(100.0), px(24.0)));
        let close_size = px(18.0);
        let padding_right = px(6.0);

        assert!(
            tab_close_hit_test(
                bounds,
                Point::new(px(20.0 + 80.0), px(10.0 + 10.0)),
                close_size,
                padding_right
            ),
            "pointer inside expected close rect should hit"
        );
        assert!(
            !tab_close_hit_test(
                bounds,
                Point::new(px(20.0 + 70.0), px(10.0 + 10.0)),
                close_size,
                padding_right
            ),
            "pointer left of expected close rect should miss"
        );
    }
}

