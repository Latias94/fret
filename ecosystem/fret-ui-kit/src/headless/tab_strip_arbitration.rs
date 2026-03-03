use fret_core::geometry::Point;
use fret_core::{Px, Rect};

pub fn tab_close_hit_test(
    bounds_local: Rect,
    pointer_local: Point,
    close_size: Px,
    padding_right: Px,
) -> bool {
    let close_size = close_size.0.max(0.0);
    let padding_right = padding_right.0.max(0.0);
    let close_x0 = bounds_local.size.width.0 - padding_right - close_size;
    let close_y0 = (bounds_local.size.height.0 - close_size) * 0.5;

    pointer_local.x.0 >= close_x0
        && pointer_local.x.0 <= close_x0 + close_size
        && pointer_local.y.0 >= close_y0
        && pointer_local.y.0 <= close_y0 + close_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::geometry::{Size, px};

    #[test]
    fn tab_close_hit_test_matches_expected_rect() {
        let bounds = Rect::new(Point::new(px(0.0), px(0.0)), Size::new(px(100.0), px(24.0)));
        let close_size = px(18.0);
        let padding_right = px(6.0);

        assert!(
            tab_close_hit_test(bounds, Point::new(px(80.0), px(10.0)), close_size, padding_right),
            "pointer inside expected close rect should hit"
        );
        assert!(
            !tab_close_hit_test(bounds, Point::new(px(70.0), px(10.0)), close_size, padding_right),
            "pointer left of expected close rect should miss"
        );
    }
}
