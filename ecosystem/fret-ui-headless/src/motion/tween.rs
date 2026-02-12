//! Time-based tween timeline (duration + easing).

use std::time::Duration;

use super::simulation::{MotionSample, secs};

/// A simple tween between two values over a fixed duration.
#[derive(Clone, Copy)]
pub struct TweenTimeline {
    start: f64,
    end: f64,
    duration: Duration,
    ease: fn(f32) -> f32,
}

impl std::fmt::Debug for TweenTimeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TweenTimeline")
            .field("start", &self.start)
            .field("end", &self.end)
            .field("duration_ms", &self.duration.as_millis())
            .finish_non_exhaustive()
    }
}

impl TweenTimeline {
    pub fn new(start: f64, end: f64, duration: Duration, ease: fn(f32) -> f32) -> Self {
        assert!(duration.as_nanos() > 0, "duration must be positive");
        Self {
            start,
            end,
            duration,
            ease,
        }
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn sample(&self, elapsed: Duration) -> MotionSample {
        let linear = if elapsed >= self.duration {
            1.0
        } else if elapsed == Duration::ZERO {
            0.0
        } else {
            (secs(elapsed) / secs(self.duration)).clamp(0.0, 1.0)
        };

        let eased = (self.ease)(linear as f32).clamp(0.0, 1.0) as f64;
        let value = self.start + (self.end - self.start) * eased;

        // Best-effort velocity estimate. This is primarily used for retargeting continuity.
        // Physics simulations provide analytic velocities; for tweens we approximate.
        let dt = Duration::from_millis(1);
        let velocity = if elapsed == Duration::ZERO {
            let next = self.sample(dt);
            (next.value - value) / secs(dt)
        } else {
            let prev = elapsed.saturating_sub(dt);
            let prev_value = self.sample(prev).value;
            (value - prev_value) / secs(dt)
        };

        MotionSample {
            value,
            velocity,
            done: elapsed >= self.duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_reaches_end() {
        let t = TweenTimeline::new(0.0, 10.0, Duration::from_millis(200), crate::easing::linear);
        let a = t.sample(Duration::ZERO);
        assert!((a.value - 0.0).abs() < 1e-9);
        let b = t.sample(Duration::from_millis(200));
        assert!((b.value - 10.0).abs() < 1e-9);
        assert!(b.done);
    }
}
