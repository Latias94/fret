use fret_core::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScrollConfig {
    /// Edge margin in px where auto-scroll starts.
    pub margin_px: f32,
    /// Minimum speed (px per tick) once inside the margin.
    pub min_speed_px_per_tick: f32,
    /// Maximum speed (px per tick).
    pub max_speed_px_per_tick: f32,
}

impl Default for AutoScrollConfig {
    fn default() -> Self {
        Self {
            margin_px: 32.0,
            min_speed_px_per_tick: 0.0,
            max_speed_px_per_tick: 16.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScrollRequest {
    pub delta: Point,
}

pub fn compute_autoscroll_x(
    config: AutoScrollConfig,
    container: Rect,
    pointer: Point,
) -> Option<Px> {
    let (margin, min_speed, max_speed) = sanitize_config(config)?;

    let left = container.origin.x.0;
    let right = left + container.size.width.0;

    let dx = if pointer.x.0 >= left && pointer.x.0 < left + margin {
        -ramp_speed(left + margin - pointer.x.0, margin, min_speed, max_speed)
    } else if pointer.x.0 <= right && pointer.x.0 > right - margin {
        ramp_speed(pointer.x.0 - (right - margin), margin, min_speed, max_speed)
    } else {
        0.0
    };

    (dx != 0.0).then_some(Px(dx))
}

pub fn compute_autoscroll_y(
    config: AutoScrollConfig,
    container: Rect,
    pointer: Point,
) -> Option<Px> {
    let (margin, min_speed, max_speed) = sanitize_config(config)?;

    let top = container.origin.y.0;
    let bottom = top + container.size.height.0;

    let dy = if pointer.y.0 >= top && pointer.y.0 < top + margin {
        -ramp_speed(top + margin - pointer.y.0, margin, min_speed, max_speed)
    } else if pointer.y.0 <= bottom && pointer.y.0 > bottom - margin {
        ramp_speed(
            pointer.y.0 - (bottom - margin),
            margin,
            min_speed,
            max_speed,
        )
    } else {
        0.0
    };

    (dy != 0.0).then_some(Px(dy))
}

pub fn compute_autoscroll(
    config: AutoScrollConfig,
    container: Rect,
    pointer: Point,
) -> Option<AutoScrollRequest> {
    let dx = compute_autoscroll_x(config, container, pointer).unwrap_or(Px(0.0));
    let dy = compute_autoscroll_y(config, container, pointer).unwrap_or(Px(0.0));
    if dx.0 == 0.0 && dy.0 == 0.0 {
        return None;
    }

    Some(AutoScrollRequest {
        delta: Point::new(dx, dy),
    })
}

fn sanitize_config(config: AutoScrollConfig) -> Option<(f32, f32, f32)> {
    let margin = if config.margin_px.is_finite() {
        config.margin_px.max(0.0)
    } else {
        0.0
    };
    let max_speed = if config.max_speed_px_per_tick.is_finite() {
        config.max_speed_px_per_tick.max(0.0)
    } else {
        0.0
    };
    let min_speed = if config.min_speed_px_per_tick.is_finite() {
        config.min_speed_px_per_tick.max(0.0)
    } else {
        0.0
    };

    let max_speed = max_speed.max(min_speed);

    (margin > 0.0 && max_speed > 0.0).then_some((margin, min_speed, max_speed))
}

fn ramp_speed(distance_into_margin: f32, margin: f32, min_speed: f32, max_speed: f32) -> f32 {
    // Normalize to [0..1], clamp, then scale linearly.
    let t = (distance_into_margin / margin).clamp(0.0, 1.0);
    min_speed + t * (max_speed - min_speed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::Size;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::new(Point::new(Px(x), Px(y)), Size::new(Px(w), Px(h)))
    }

    #[test]
    fn no_scroll_when_not_near_edges() {
        let cfg = AutoScrollConfig {
            margin_px: 10.0,
            min_speed_px_per_tick: 0.0,
            max_speed_px_per_tick: 10.0,
        };
        assert_eq!(
            compute_autoscroll(
                cfg,
                rect(0.0, 0.0, 100.0, 100.0),
                Point::new(Px(50.0), Px(50.0))
            ),
            None
        );
    }

    #[test]
    fn scrolls_towards_nearest_edge() {
        let cfg = AutoScrollConfig {
            margin_px: 10.0,
            min_speed_px_per_tick: 0.0,
            max_speed_px_per_tick: 10.0,
        };
        let req = compute_autoscroll(
            cfg,
            rect(0.0, 0.0, 100.0, 100.0),
            Point::new(Px(2.0), Px(50.0)),
        )
        .expect("should scroll");
        assert!(req.delta.x.0 < 0.0);
        assert_eq!(req.delta.y.0, 0.0);
    }

    #[test]
    fn min_speed_applies_within_margin() {
        let cfg = AutoScrollConfig {
            margin_px: 10.0,
            min_speed_px_per_tick: 5.0,
            max_speed_px_per_tick: 10.0,
        };
        assert_eq!(
            compute_autoscroll_x(
                cfg,
                rect(0.0, 0.0, 100.0, 100.0),
                Point::new(Px(10.0), Px(50.0))
            ),
            None,
            "exactly at the margin boundary should not scroll"
        );
        let dx = compute_autoscroll_x(
            cfg,
            rect(0.0, 0.0, 100.0, 100.0),
            Point::new(Px(9.0), Px(50.0)),
        )
        .expect("should scroll inside the margin");
        assert!(dx.0 < 0.0);
        assert!(dx.0.abs() >= 5.0);
    }
}
