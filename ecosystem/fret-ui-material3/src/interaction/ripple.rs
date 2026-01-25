//! Ripple animation policy (Material-like).
//!
//! This is a policy-level helper that drives a single ripple pulse animation. Rendering is
//! performed by `fret_ui::paint::paint_ripple` (mechanism-level primitive).

use fret_core::{Point, Px};
use fret_ui::theme::CubicBezier;

use crate::motion::{cubic_bezier_ease, ms_to_frames};

#[derive(Debug, Clone, Copy)]
pub enum RippleOrigin {
    /// Origin in the scene coordinate space.
    Absolute(Point),
    /// Origin relative to the paint bounds origin.
    Local(Point),
}

#[derive(Debug, Clone, Copy)]
pub struct RipplePaintFrame {
    pub origin: RippleOrigin,
    pub radius: Px,
    pub color: fret_core::Color,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy)]
struct RipplePulse {
    origin: RippleOrigin,
    max_radius: Px,
    color: fret_core::Color,
    start_frame: u64,
    expand_frames: u64,
    release_frame: Option<u64>,
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
        origin: RippleOrigin,
        max_radius: Px,
        color: fret_core::Color,
        expand_duration_ms: u32,
        fade_duration_ms: u32,
        easing: CubicBezier,
    ) {
        let expand_frames = ms_to_frames(expand_duration_ms).max(1);
        let fade_frames = ms_to_frames(fade_duration_ms).max(1);
        self.active = Some(RipplePulse {
            origin,
            max_radius,
            color,
            start_frame: now_frame,
            expand_frames,
            release_frame: None,
            fade_frames,
            easing,
        });
    }

    pub fn release(&mut self, now_frame: u64) {
        let Some(mut pulse) = self.active else {
            return;
        };
        if pulse.release_frame.is_some() {
            return;
        }
        pulse.release_frame = Some(now_frame);
        self.active = Some(pulse);
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

        let opacity = match pulse.release_frame {
            None => base_opacity,
            Some(release_frame) => {
                let fade_elapsed = now_frame.saturating_sub(release_frame).saturating_add(1);
                let fade_t = (fade_elapsed as f32 / pulse.fade_frames as f32).clamp(0.0, 1.0);
                let opacity = (base_opacity * (1.0 - fade_t)).clamp(0.0, 1.0);
                if fade_t >= 1.0 {
                    self.active = None;
                    return None;
                }
                opacity
            }
        };

        Some(RipplePaintFrame {
            origin: pulse.origin,
            radius,
            color: pulse.color,
            opacity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::RippleAnimator;
    use super::RippleOrigin;
    use fret_core::{Point, Px};
    use fret_ui::theme::CubicBezier;

    #[test]
    fn ripple_animator_finishes_after_fade() {
        let mut a = RippleAnimator::default();
        a.start(
            0,
            RippleOrigin::Absolute(Point::new(Px(0.0), Px(0.0))),
            Px(10.0),
            fret_core::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
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
        for f in 0..20 {
            last = a.advance(f, 0.12);
        }
        a.release(20);
        for f in 20..200 {
            last = a.advance(f, 0.12);
        }
        // After enough frames, it should be inactive.
        let _ = last;
        let _ = a.advance(10_000, 0.12);
        assert!(!a.is_active());
    }

    #[test]
    fn ripple_does_not_fade_until_release() {
        let mut a = RippleAnimator::default();
        a.start(
            0,
            RippleOrigin::Absolute(Point::new(Px(0.0), Px(0.0))),
            Px(10.0),
            fret_core::Color {
                r: 1.0,
                g: 1.0,
                b: 1.0,
                a: 1.0,
            },
            100,
            100,
            CubicBezier {
                x1: 0.0,
                y1: 0.0,
                x2: 1.0,
                y2: 1.0,
            },
        );

        for f in 0..40 {
            let frame = a.advance(f, 0.12).expect("expected active ripple");
            assert!(
                (frame.opacity - 0.12).abs() < 1e-6,
                "ripple should not fade while held"
            );
        }

        a.release(40);
        let f41 = a.advance(41, 0.12).expect("expected active ripple");
        assert!(
            f41.opacity < 0.12,
            "ripple should start fading after release"
        );
    }
}
