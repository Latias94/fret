//! Easing helpers for deterministic motion.
//!
//! Fret's UI motion is driven by a monotonic tick/frame clock (ADR 0034). shadcn/ui and Radix
//! often express easing in CSS (e.g. `ease-[cubic-bezier(0.22,1,0.36,1)]`). This module provides
//! small, reusable easing math so motion policies can match those outcomes without DOM/CSS.

/// A CSS-style cubic-bezier easing curve.
///
/// This models the common CSS `cubic-bezier(x1, y1, x2, y2)` definition where the curve starts at
/// (0, 0) and ends at (1, 1). For typical easing curves, `x(t)` is monotonic which allows us to
/// solve `t` for an input progress `x` and then sample `y(t)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Sample the easing output `y` for an input progress `x` in `[0, 1]`.
    pub fn sample(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }

        // Solve for parameter t where curve_x(t) ~= x.
        let t = self.solve_t_for_x(x);
        self.curve_y(t).clamp(0.0, 1.0)
    }

    /// Sample the easing output `y` for an input progress `x` in `[0, 1]` without clamping.
    ///
    /// This is useful for "overshoot" curves like `cubic-bezier(..., y2 > 1.0)` where the
    /// desired output intentionally goes outside `[0, 1]`.
    pub fn sample_unclamped(self, x: f32) -> f32 {
        let x = x.clamp(0.0, 1.0);
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }

        let t = self.solve_t_for_x(x);
        self.curve_y(t)
    }

    fn curve_x(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        3.0 * u * u * t * self.x1 + 3.0 * u * t * t * self.x2 + t * t * t
    }

    fn curve_y(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        3.0 * u * u * t * self.y1 + 3.0 * u * t * t * self.y2 + t * t * t
    }

    fn curve_dx_dt(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let u = 1.0 - t;
        // Derivative of the cubic bezier x(t) with endpoints fixed at 0 and 1.
        3.0 * u * u * self.x1 + 6.0 * u * t * (self.x2 - self.x1) + 3.0 * t * t * (1.0 - self.x2)
    }

    fn solve_t_for_x(self, x: f32) -> f32 {
        // First try a small fixed number of Newton steps (fast).
        let mut t = x;
        for _ in 0..8 {
            let x_t = self.curve_x(t);
            let dx = x_t - x;
            if dx.abs() < 1e-6 {
                return t.clamp(0.0, 1.0);
            }
            let d = self.curve_dx_dt(t);
            if d.abs() < 1e-6 {
                break;
            }
            t -= dx / d;
            if !(0.0..=1.0).contains(&t) {
                break;
            }
        }

        // Fall back to bisection in [0, 1] (robust).
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        for _ in 0..24 {
            let mid = 0.5 * (lo + hi);
            let x_mid = self.curve_x(mid);
            if (x_mid - x).abs() < 1e-6 {
                return mid;
            }
            if x_mid < x {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }
}

pub const EASE: CubicBezier = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
pub const EASE_IN: CubicBezier = CubicBezier::new(0.42, 0.0, 1.0, 1.0);
pub const EASE_OUT: CubicBezier = CubicBezier::new(0.0, 0.0, 0.58, 1.0);
pub const EASE_IN_OUT: CubicBezier = CubicBezier::new(0.42, 0.0, 0.58, 1.0);

/// shadcn/ui v4 commonly uses `ease-[cubic-bezier(0.22,1,0.36,1)]` for overlay-like transitions.
pub const SHADCN_EASE: CubicBezier = CubicBezier::new(0.22, 1.0, 0.36, 1.0);

pub fn linear(t: f32) -> f32 {
    t.clamp(0.0, 1.0)
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cubic_bezier_hits_endpoints() {
        let c = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
        assert_eq!(c.sample(0.0), 0.0);
        assert_eq!(c.sample(1.0), 1.0);
    }

    #[test]
    fn cubic_bezier_linear_is_identity() {
        let c = CubicBezier::new(0.0, 0.0, 1.0, 1.0);
        for i in 0..=10 {
            let x = i as f32 / 10.0;
            let y = c.sample(x);
            assert!((y - x).abs() < 1e-4, "x={x} y={y}");
        }
    }

    #[test]
    fn cubic_bezier_is_monotonic_for_common_presets() {
        let curves = [EASE, EASE_IN, EASE_OUT, EASE_IN_OUT, SHADCN_EASE];
        for c in curves {
            let mut prev = 0.0f32;
            for i in 0..=100 {
                let x = i as f32 / 100.0;
                let y = c.sample(x);
                assert!(y >= prev - 1e-4, "curve={c:?} x={x} prev={prev} y={y}");
                prev = y;
            }
        }
    }

    #[test]
    fn cubic_bezier_sample_unclamped_allows_overshoot() {
        let c = CubicBezier::new(0.175, 0.885, 0.32, 1.275);
        let mut seen_overshoot = false;
        for i in 0..=100 {
            let x = i as f32 / 100.0;
            let y = c.sample_unclamped(x);
            if y > 1.0 {
                seen_overshoot = true;
                break;
            }
        }
        assert!(seen_overshoot);
    }
}
