use fret_core::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

pub fn axis_lock(translation: Point, axis: Axis) -> Point {
    match axis {
        Axis::X => Point::new(translation.x, Px(0.0)),
        Axis::Y => Point::new(Px(0.0), translation.y),
    }
}

pub fn clamp_rect_translation(rect: Rect, translation: Point, bounds: Rect) -> Point {
    let min_x = bounds.origin.x.0;
    let min_y = bounds.origin.y.0;
    let max_x = (min_x + bounds.size.width.0 - rect.size.width.0).max(min_x);
    let max_y = (min_y + bounds.size.height.0 - rect.size.height.0).max(min_y);

    let desired_x = rect.origin.x.0 + translation.x.0;
    let desired_y = rect.origin.y.0 + translation.y.0;

    let clamped_x = desired_x.clamp(min_x, max_x);
    let clamped_y = desired_y.clamp(min_y, max_y);

    Point::new(
        Px(clamped_x - rect.origin.x.0),
        Px(clamped_y - rect.origin.y.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn axis_lock_zeroes_other_component() {
        let t = Point::new(Px(5.0), Px(3.0));
        assert_eq!(axis_lock(t, Axis::X), Point::new(Px(5.0), Px(0.0)));
        assert_eq!(axis_lock(t, Axis::Y), Point::new(Px(0.0), Px(3.0)));
    }

    #[test]
    fn clamp_translation_keeps_rect_within_bounds() {
        let bounds = rect(0.0, 0.0, 100.0, 100.0);
        let item = rect(10.0, 10.0, 20.0, 20.0);

        let t = clamp_rect_translation(item, Point::new(Px(1000.0), Px(0.0)), bounds);
        assert_eq!(t, Point::new(Px(70.0), Px(0.0)));
    }

    #[test]
    fn clamp_translation_does_not_panic_when_rect_is_larger_than_bounds() {
        let bounds = rect(0.0, 0.0, 10.0, 10.0);
        let item = rect(5.0, 5.0, 20.0, 20.0);

        let t = clamp_rect_translation(item, Point::new(Px(1000.0), Px(1000.0)), bounds);
        assert_eq!(t, Point::new(Px(-5.0), Px(-5.0)));
    }
}
