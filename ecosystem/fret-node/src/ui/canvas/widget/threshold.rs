mod distance;
mod normalize;

use fret_core::Point;

pub(super) fn exceeds_drag_threshold(
    start: Point,
    position: Point,
    threshold_screen: f32,
    zoom: f32,
) -> bool {
    let threshold_screen = normalize::normalized_threshold_screen(threshold_screen);

    if threshold_screen <= 0.0 {
        return true;
    }

    distance::distance2(start, position)
        >= normalize::threshold_canvas_units(threshold_screen, zoom).powi(2)
}

#[cfg(test)]
mod tests {
    use super::exceeds_drag_threshold;
    use fret_core::{Point, Px};

    #[test]
    fn threshold_zero_always_exceeds() {
        let start = Point::new(Px(0.0), Px(0.0));
        let pos = Point::new(Px(0.0), Px(0.0));
        assert!(exceeds_drag_threshold(start, pos, 0.0, 1.0));
        assert!(exceeds_drag_threshold(start, pos, -10.0, 1.0));
    }

    #[test]
    fn threshold_checks_screen_distance() {
        let start = Point::new(Px(0.0), Px(0.0));
        assert!(!exceeds_drag_threshold(
            start,
            Point::new(Px(7.9), Px(0.0)),
            8.0,
            1.0
        ));
        assert!(exceeds_drag_threshold(
            start,
            Point::new(Px(8.0), Px(0.0)),
            8.0,
            1.0
        ));
        assert!(exceeds_drag_threshold(
            start,
            Point::new(Px(0.0), Px(8.0)),
            8.0,
            1.0
        ));
    }

    #[test]
    fn threshold_is_zoom_invariant_in_screen_space() {
        let start = Point::new(Px(0.0), Px(0.0));
        assert!(!exceeds_drag_threshold(
            start,
            Point::new(Px(3.9), Px(0.0)),
            8.0,
            2.0
        ));
        assert!(exceeds_drag_threshold(
            start,
            Point::new(Px(4.0), Px(0.0)),
            8.0,
            2.0
        ));
    }
}
