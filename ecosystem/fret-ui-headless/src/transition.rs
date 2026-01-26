//! Transition timelines (deterministic, tick-driven).
//!
//! This module provides a small, deterministic state machine for UI transitions that need:
//!
//! - different open/close durations,
//! - a stable `present` vs unmounted outcome (keep mounted while closing),
//! - a normalized progress value (`0..1`) that can be eased.
//!
//! It is intended to be driven by a monotonic tick source (typically a frame counter).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hidden,
    Opening { start_tick: u64 },
    Open,
    Closing { start_tick: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransitionOutput {
    /// Whether the content should remain mounted/paintable.
    pub present: bool,
    /// Linear progress in `[0, 1]` (0 = fully closed, 1 = fully open).
    pub linear: f32,
    /// Eased progress in `[0, 1]` using the easing function passed to update.
    pub progress: f32,
    /// Whether the transition is currently animating.
    pub animating: bool,
}

/// A tiny open/close transition timeline.
#[derive(Debug, Clone, Copy)]
pub struct TransitionTimeline {
    open_ticks: u64,
    close_ticks: u64,
    phase: Phase,
}

impl Default for TransitionTimeline {
    fn default() -> Self {
        Self {
            open_ticks: 4,
            close_ticks: 4,
            phase: Phase::Hidden,
        }
    }
}

impl TransitionTimeline {
    pub fn open_ticks(&self) -> u64 {
        self.open_ticks
    }

    pub fn close_ticks(&self) -> u64 {
        self.close_ticks
    }

    pub fn set_open_ticks(&mut self, open_ticks: u64) {
        self.open_ticks = open_ticks.max(1);
    }

    pub fn set_close_ticks(&mut self, close_ticks: u64) {
        self.close_ticks = close_ticks.max(1);
    }

    pub fn set_durations(&mut self, open_ticks: u64, close_ticks: u64) {
        self.open_ticks = open_ticks.max(1);
        self.close_ticks = close_ticks.max(1);
    }

    pub fn update(&mut self, open: bool, tick: u64) -> TransitionOutput {
        self.update_with_easing(open, tick, crate::easing::smoothstep)
    }

    pub fn update_with_easing(
        &mut self,
        open: bool,
        tick: u64,
        ease: fn(f32) -> f32,
    ) -> TransitionOutput {
        if open {
            match self.phase {
                Phase::Hidden | Phase::Closing { .. } => {
                    self.phase = Phase::Opening { start_tick: tick };
                }
                Phase::Opening { .. } | Phase::Open => {}
            }
        } else {
            match self.phase {
                Phase::Open | Phase::Opening { .. } => {
                    self.phase = Phase::Closing { start_tick: tick };
                }
                Phase::Closing { .. } | Phase::Hidden => {}
            }
        }

        match self.phase {
            Phase::Hidden => TransitionOutput {
                present: false,
                linear: 0.0,
                progress: 0.0,
                animating: false,
            },
            Phase::Open => TransitionOutput {
                present: true,
                linear: 1.0,
                progress: 1.0,
                animating: false,
            },
            Phase::Opening { start_tick } => {
                let duration = self.open_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / duration as f32).clamp(0.0, 1.0);
                let linear = t;
                let progress = ease(linear).clamp(0.0, 1.0);
                if t >= 1.0 {
                    self.phase = Phase::Open;
                    TransitionOutput {
                        present: true,
                        linear: 1.0,
                        progress: 1.0,
                        animating: false,
                    }
                } else {
                    TransitionOutput {
                        present: true,
                        linear,
                        progress,
                        animating: true,
                    }
                }
            }
            Phase::Closing { start_tick } => {
                let duration = self.close_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / duration as f32).clamp(0.0, 1.0);
                let linear = (1.0 - t).clamp(0.0, 1.0);
                let progress = ease(linear).clamp(0.0, 1.0);
                if t >= 1.0 {
                    self.phase = Phase::Hidden;
                    TransitionOutput {
                        present: false,
                        linear: 0.0,
                        progress: 0.0,
                        animating: false,
                    }
                } else {
                    TransitionOutput {
                        present: true,
                        linear,
                        progress,
                        animating: true,
                    }
                }
            }
        }
    }

    pub fn update_with_cubic_bezier(
        &mut self,
        open: bool,
        tick: u64,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    ) -> TransitionOutput {
        if open {
            match self.phase {
                Phase::Hidden | Phase::Closing { .. } => {
                    self.phase = Phase::Opening { start_tick: tick };
                }
                Phase::Opening { .. } | Phase::Open => {}
            }
        } else {
            match self.phase {
                Phase::Open | Phase::Opening { .. } => {
                    self.phase = Phase::Closing { start_tick: tick };
                }
                Phase::Closing { .. } | Phase::Hidden => {}
            }
        }

        match self.phase {
            Phase::Hidden => TransitionOutput {
                present: false,
                linear: 0.0,
                progress: 0.0,
                animating: false,
            },
            Phase::Open => TransitionOutput {
                present: true,
                linear: 1.0,
                progress: 1.0,
                animating: false,
            },
            Phase::Opening { start_tick } => {
                let duration = self.open_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / duration as f32).clamp(0.0, 1.0);
                let linear = t;
                let progress = cubic_bezier_ease(x1, y1, x2, y2, linear).clamp(0.0, 1.0);
                if t >= 1.0 {
                    self.phase = Phase::Open;
                    TransitionOutput {
                        present: true,
                        linear: 1.0,
                        progress: 1.0,
                        animating: false,
                    }
                } else {
                    TransitionOutput {
                        present: true,
                        linear,
                        progress,
                        animating: true,
                    }
                }
            }
            Phase::Closing { start_tick } => {
                let duration = self.close_ticks.max(1);
                let elapsed = tick.saturating_sub(start_tick).saturating_add(1);
                let t = (elapsed as f32 / duration as f32).clamp(0.0, 1.0);
                let linear = (1.0 - t).clamp(0.0, 1.0);
                let progress = cubic_bezier_ease(x1, y1, x2, y2, linear).clamp(0.0, 1.0);
                if t >= 1.0 {
                    self.phase = Phase::Hidden;
                    TransitionOutput {
                        present: false,
                        linear: 0.0,
                        progress: 0.0,
                        animating: false,
                    }
                } else {
                    TransitionOutput {
                        present: true,
                        linear,
                        progress,
                        animating: true,
                    }
                }
            }
        }
    }
}

fn cubic_bezier_ease(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);

    if (x1, y1, x2, y2) == (0.0, 0.0, 1.0, 1.0) {
        return t;
    }

    let mut u = t;
    for _ in 0..8 {
        let x = cubic_bezier_x(x1, x2, u);
        let dx = cubic_bezier_x_derivative(x1, x2, u);
        if dx.abs() < 1e-6 {
            break;
        }
        u = (u - (x - t) / dx).clamp(0.0, 1.0);
    }

    let mut lo = 0.0;
    let mut hi = 1.0;
    for _ in 0..12 {
        let x = cubic_bezier_x(x1, x2, u);
        if (x - t).abs() < 1e-4 {
            break;
        }
        if x < t {
            lo = u;
        } else {
            hi = u;
        }
        u = (lo + hi) * 0.5;
    }

    cubic_bezier_y(y1, y2, u).clamp(0.0, 1.0)
}

fn cubic_bezier_x(p1: f32, p2: f32, u: f32) -> f32 {
    cubic_bezier_component(u, p1, p2)
}

fn cubic_bezier_y(p1: f32, p2: f32, u: f32) -> f32 {
    cubic_bezier_component(u, p1, p2)
}

fn cubic_bezier_x_derivative(p1: f32, p2: f32, u: f32) -> f32 {
    cubic_bezier_component_derivative(u, p1, p2)
}

fn cubic_bezier_component(u: f32, p1: f32, p2: f32) -> f32 {
    let inv = 1.0 - u;
    3.0 * inv * inv * u * p1 + 3.0 * inv * u * u * p2 + u * u * u
}

fn cubic_bezier_component_derivative(u: f32, p1: f32, p2: f32) -> f32 {
    let inv = 1.0 - u;
    3.0 * inv * inv * p1 + 6.0 * inv * u * (p2 - p1) + 3.0 * u * u * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opens_and_closes_with_present_window() {
        let mut t = TransitionTimeline::default();
        t.set_durations(3, 3);

        let o0 = t.update(true, 0);
        assert!(o0.present);
        assert!(o0.animating);
        assert!(o0.linear > 0.0 && o0.linear < 1.0);

        let o2 = t.update(true, 2);
        assert!(o2.present);

        let o3 = t.update(true, 3);
        assert!(o3.present);
        assert!(!o3.animating);
        assert_eq!(o3.linear, 1.0);

        let c0 = t.update(false, 4);
        assert!(c0.present);
        assert!(c0.animating);
        assert!(c0.linear < 1.0);

        let c3 = t.update(false, 7);
        assert!(!c3.present);
        assert!(!c3.animating);
        assert_eq!(c3.linear, 0.0);
    }

    #[test]
    fn can_use_shadcn_cubic_bezier_easing() {
        let mut t = TransitionTimeline::default();
        t.set_durations(4, 4);
        let out = t.update_with_easing(true, 0, |x| crate::easing::SHADCN_EASE.sample(x));
        assert!(out.present);
        assert!(out.animating);
        assert!(out.progress >= 0.0 && out.progress <= 1.0);
    }

    #[test]
    fn cubic_bezier_transition_matches_linear_for_linear_curve() {
        let mut t = TransitionTimeline::default();
        t.set_durations(4, 4);
        let out = t.update_with_cubic_bezier(true, 0, 0.0, 0.0, 1.0, 1.0);
        assert!(out.present);
        assert!(out.animating);
        assert!((out.progress - out.linear).abs() <= 1e-3);
    }
}
