use fret_core::Point;

pub(super) fn exceeds_drag_threshold(start: Point, position: Point, threshold_screen: f32) -> bool {
    let t = if threshold_screen.is_finite() {
        threshold_screen.max(0.0)
    } else {
        0.0
    };

    if t <= 0.0 {
        return true;
    }

    let dx = position.x.0 - start.x.0;
    let dy = position.y.0 - start.y.0;
    dx * dx + dy * dy >= t * t
}

#[cfg(test)]
mod tests {
    use super::exceeds_drag_threshold;
    use fret_core::{Point, Px};

    #[test]
    fn threshold_zero_always_exceeds() {
        let start = Point::new(Px(0.0), Px(0.0));
        let pos = Point::new(Px(0.0), Px(0.0));
        assert!(exceeds_drag_threshold(start, pos, 0.0));
        assert!(exceeds_drag_threshold(start, pos, -10.0));
    }

    #[test]
    fn threshold_checks_screen_distance() {
        let start = Point::new(Px(0.0), Px(0.0));
        assert!(!exceeds_drag_threshold(
            start,
            Point::new(Px(7.9), Px(0.0)),
            8.0
        ));
        assert!(exceeds_drag_threshold(
            start,
            Point::new(Px(8.0), Px(0.0)),
            8.0
        ));
        assert!(exceeds_drag_threshold(
            start,
            Point::new(Px(0.0), Px(8.0)),
            8.0
        ));
    }
}
