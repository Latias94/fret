//! Ripple animation policy (Material-like).
//!
//! This is a policy-level helper that drives a single ripple pulse animation. Rendering is
//! performed by `fret_ui::paint::paint_ripple` (mechanism-level primitive).

use fret_core::{Point, Px};
use fret_ui::theme::CubicBezier;

use crate::motion::{cubic_bezier_ease, ms_to_frames};

#[derive(Debug, Clone, Copy)]
pub struct RipplePaintFrame {
    pub origin: Point,
    pub radius: Px,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy)]
struct RipplePulse {
    origin: Point,
    max_radius: Px,
    start_frame: u64,
    expand_frames: u64,
    fade_start_frame: u64,
    fade_frames: u64,
    easing: CubicBezier,
}

#[derive(Debug, Default, Clone)]
pub struct RippleAnimator {
    active: Option<RipplePulse>,
}

impl RippleAnimator {
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    pub fn start(
        &mut self,
        now_frame: u64,
        origin: Point,
        max_radius: Px,
        expand_duration_ms: u32,
        fade_duration_ms: u32,
        easing: CubicBezier,
    ) {
        let expand_frames = ms_to_frames(expand_duration_ms).max(1);
        let fade_frames = ms_to_frames(fade_duration_ms).max(1);
        self.active = Some(RipplePulse {
            origin,
            max_radius,
            start_frame: now_frame,
            expand_frames,
            fade_start_frame: now_frame.saturating_add(expand_frames),
            fade_frames,
            easing,
        });
    }

    pub fn advance(&mut self, now_frame: u64, base_opacity: f32) -> Option<RipplePaintFrame> {
        let pulse = self.active?;

        // Ensure the first rendered frame has a non-zero radius so the ripple becomes visible
        // immediately on press (instead of being delayed by one frame).
        let expand_elapsed = now_frame
            .saturating_sub(pulse.start_frame)
            .saturating_add(1);
        let expand_t = (expand_elapsed as f32 / pulse.expand_frames as f32).clamp(0.0, 1.0);
        let expand_e = cubic_bezier_ease(pulse.easing, expand_t);
        let radius = Px(pulse.max_radius.0 * expand_e);

        let fade_elapsed = now_frame.saturating_sub(pulse.fade_start_frame);
        let fade_t = (fade_elapsed as f32 / pulse.fade_frames as f32).clamp(0.0, 1.0);
        let opacity = (base_opacity * (1.0 - fade_t)).clamp(0.0, 1.0);

        if fade_t >= 1.0 {
            self.active = None;
            return None;
        }

        Some(RipplePaintFrame {
            origin: pulse.origin,
            radius,
            opacity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RippleAnimator;
    use fret_core::{Point, Px};
    use fret_ui::theme::CubicBezier;

    #[test]
    fn ripple_animator_finishes_after_fade() {
        let mut a = RippleAnimator::default();
        a.start(
            0,
            Point::new(Px(0.0), Px(0.0)),
            Px(10.0),
            100,
            100,
            CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        );
        assert!(a.is_active());
        let mut last = None;
        for f in 0..200 {
            last = a.advance(f, 0.12);
        }
        // After enough frames, it should be inactive.
        let _ = last;
        let _ = a.advance(10_000, 0.12);
        assert!(!a.is_active());
    }
}
