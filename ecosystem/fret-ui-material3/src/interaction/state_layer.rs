//! State layer animation policy (Material-like).
//!
//! This is a policy-level helper that computes an animated state-layer opacity. Rendering is
//! performed by `fret_ui::paint::paint_state_layer` (mechanism-level primitive).

use fret_ui::theme::CubicBezier;

use crate::motion::{cubic_bezier_ease, ms_to_frames};

#[derive(Debug, Clone, Copy)]
pub struct StateLayerAnimator {
    current: f32,
    from: f32,
    to: f32,
    start_frame: u64,
    duration_frames: u64,
    easing: CubicBezier,
    active: bool,
}

impl Default for StateLayerAnimator {
    fn default() -> Self {
        Self {
            current: 0.0,
            from: 0.0,
            to: 0.0,
            start_frame: 0,
            duration_frames: 0,
            easing: CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
            active: false,
        }
    }
}

impl StateLayerAnimator {
    pub fn value(&self) -> f32 {
        self.current
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_target(
        &mut self,
        now_frame: u64,
        target: f32,
        duration_ms: u32,
        easing: CubicBezier,
    ) {
        let target = target.clamp(0.0, 1.0);
        if (target - self.to).abs() < 1e-6 && !self.active {
            self.current = target;
            self.from = target;
            self.to = target;
            return;
        }

        self.from = self.current;
        self.to = target;
        self.start_frame = now_frame;
        self.duration_frames = ms_to_frames(duration_ms).max(1);
        self.easing = easing;
        self.active = true;
    }

    pub fn advance(&mut self, now_frame: u64) {
        if !self.active {
            return;
        }

        let elapsed = now_frame.saturating_sub(self.start_frame);
        if elapsed >= self.duration_frames {
            self.current = self.to;
            self.active = false;
            return;
        }

        let t = elapsed as f32 / self.duration_frames as f32;
        let e = cubic_bezier_ease(self.easing, t);
        self.current = self.from + (self.to - self.from) * e;
    }
}

#[cfg(test)]
mod tests {
    use super::StateLayerAnimator;
    use fret_ui::theme::CubicBezier;

    #[test]
    fn state_layer_animator_reaches_target() {
        let mut a = StateLayerAnimator::default();
        a.set_target(
            10,
            0.5,
            100,
            CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        );
        for f in 10..100 {
            a.advance(f);
        }
        a.advance(999);
        assert!(!a.is_active());
        assert!((a.value() - 0.5).abs() < 1e-4);
    }
}
