use fret_core::{Point, Px};

use super::{WinitAppDriver, WinitRunner};

fn wheel_coalesce_axis_px(prev: Px, next: Px) -> Px {
    let prev_v = prev.0;
    let next_v = next.0;
    if prev_v.signum() == next_v.signum() {
        Px(prev_v + next_v)
    } else {
        Px(next_v)
    }
}

pub(super) fn wheel_coalesce_delta(prev: Point, next: Point) -> Point {
    Point::new(
        wheel_coalesce_axis_px(prev.x, next.x),
        wheel_coalesce_axis_px(prev.y, next.y),
    )
}

fn wheel_split_axis_by_max_abs_px(delta: Px, max_abs: f32) -> (Px, Px) {
    let v = delta.0;
    if v.abs() <= max_abs {
        return (delta, Px(0.0));
    }
    let delivered = Px(v.signum() * max_abs);
    let remainder = Px(v - delivered.0);
    (delivered, remainder)
}

pub(super) fn wheel_split_delta_by_max_abs_px(delta: Point, max_abs: f32) -> (Point, Point) {
    let (dx, rx) = wheel_split_axis_by_max_abs_px(delta.x, max_abs);
    let (dy, ry) = wheel_split_axis_by_max_abs_px(delta.y, max_abs);
    (Point::new(dx, dy), Point::new(rx, ry))
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn wheel_coalescing_enabled() -> bool {
        std::env::var_os("FRET_WINIT_COALESCE_WHEEL").is_some_and(|v| !v.is_empty() && v != "0")
    }

    pub(super) fn wheel_coalescing_max_abs_px() -> f32 {
        std::env::var("FRET_WINIT_COALESCE_WHEEL_MAX_ABS_PX")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(120.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wheel_coalesce_axis_overrides_opposite_signs() {
        assert_eq!(wheel_coalesce_axis_px(Px(-10.0), Px(-5.0)), Px(-15.0));
        assert_eq!(wheel_coalesce_axis_px(Px(-10.0), Px(5.0)), Px(5.0));
        assert_eq!(wheel_coalesce_axis_px(Px(10.0), Px(-5.0)), Px(-5.0));
    }

    #[test]
    fn wheel_split_axis_by_max_abs_caps_and_carries_remainder() {
        let (delivered, remainder) = wheel_split_axis_by_max_abs_px(Px(-200.0), 120.0);
        assert_eq!(delivered, Px(-120.0));
        assert_eq!(remainder, Px(-80.0));

        let (delivered, remainder) = wheel_split_axis_by_max_abs_px(Px(50.0), 120.0);
        assert_eq!(delivered, Px(50.0));
        assert_eq!(remainder, Px(0.0));
    }

    #[test]
    fn wheel_split_delta_by_max_abs_caps_per_axis() {
        let (delivered, remainder) =
            wheel_split_delta_by_max_abs_px(Point::new(Px(-200.0), Px(50.0)), 120.0);
        assert_eq!(delivered, Point::new(Px(-120.0), Px(50.0)));
        assert_eq!(remainder, Point::new(Px(-80.0), Px(0.0)));
    }
}
