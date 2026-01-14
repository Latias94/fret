use fret_core::{Point, Px, Rect};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScrollConfig {
    /// Edge margin in px where auto-scroll starts.
    pub margin_px: f32,
    /// Maximum speed (px per tick).
    pub max_speed_px_per_tick: f32,
}

impl Default for AutoScrollConfig {
    fn default() -> Self {
        Self {
            margin_px: 32.0,
            max_speed_px_per_tick: 16.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScrollRequest {
    pub delta: Point,
}

pub fn compute_autoscroll(
    config: AutoScrollConfig,
    container: Rect,
    pointer: Point,
) -> Option<AutoScrollRequest> {
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
    if margin <= 0.0 || max_speed <= 0.0 {
        return None;
    }

    let left = container.origin.x.0;
    let top = container.origin.y.0;
    let right = left + container.size.width.0;
    let bottom = top + container.size.height.0;

    let mut dx = 0.0;
    if pointer.x.0 >= left && pointer.x.0 < left + margin {
        dx = -ramp_speed(left + margin - pointer.x.0, margin, max_speed);
    } else if pointer.x.0 <= right && pointer.x.0 > right - margin {
        dx = ramp_speed(pointer.x.0 - (right - margin), margin, max_speed);
    }

    let mut dy = 0.0;
    if pointer.y.0 >= top && pointer.y.0 < top + margin {
        dy = -ramp_speed(top + margin - pointer.y.0, margin, max_speed);
    } else if pointer.y.0 <= bottom && pointer.y.0 > bottom - margin {
        dy = ramp_speed(pointer.y.0 - (bottom - margin), margin, max_speed);
    }

    if dx == 0.0 && dy == 0.0 {
        return None;
    }

    Some(AutoScrollRequest {
        delta: Point::new(Px(dx), Px(dy)),
    })
}

fn ramp_speed(distance_into_margin: f32, margin: f32, max_speed: f32) -> f32 {
    // Normalize to [0..1], clamp, then scale linearly.
    let t = (distance_into_margin / margin).clamp(0.0, 1.0);
    t * max_speed
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
}
