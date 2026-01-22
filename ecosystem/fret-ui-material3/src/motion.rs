//! Motion helpers for Material 3 interactions.
//!
//! This module provides small, deterministic utilities for driving per-frame animations from
//! duration + cubic-bezier tokens. It intentionally avoids using wall-clock time; Fret's
//! declarative UI is typically driven by `FrameId` (monotonic frame counter).

use fret_ui::theme::CubicBezier;

pub fn ms_to_frames(ms: u32) -> u64 {
    // Match the patterns used in existing ecosystem code: assume 60Hz for deterministic tests.
    // ceil(ms * 60 / 1000)
    (ms as u64 * 60).saturating_add(999) / 1000
}

/// Evaluate a CSS-like cubic-bezier easing curve at normalized time `t`.
///
/// This returns `y` for an input `x = t` by numerically inverting the parametric bezier.
pub fn cubic_bezier_ease(bezier: CubicBezier, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);

    // Fast path for linear.
    if bezier
        == (CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        })
    {
        return t;
    }

    // Solve x(u) = t for u in [0,1], then return y(u).
    let mut u = t;
    for _ in 0..8 {
        let x = bezier_x(bezier, u);
        let dx = bezier_x_derivative(bezier, u);
        if dx.abs() < 1e-6 {
            break;
        }
        let next = u - (x - t) / dx;
        u = next.clamp(0.0, 1.0);
    }

    // Fall back to a binary search if Newton isn't stable (e.g. near-flat slopes).
    let mut lo = 0.0;
    let mut hi = 1.0;
    for _ in 0..12 {
        let x = bezier_x(bezier, u);
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

    bezier_y(bezier, u).clamp(0.0, 1.0)
}

fn bezier_x(bezier: CubicBezier, u: f32) -> f32 {
    bezier_component(u, bezier.x1, bezier.x2)
}

fn bezier_y(bezier: CubicBezier, u: f32) -> f32 {
    bezier_component(u, bezier.y1, bezier.y2)
}

fn bezier_x_derivative(bezier: CubicBezier, u: f32) -> f32 {
    bezier_component_derivative(u, bezier.x1, bezier.x2)
}

fn bezier_component(u: f32, p1: f32, p2: f32) -> f32 {
    // Cubic bezier with P0=0, P1=p1, P2=p2, P3=1.
    let inv = 1.0 - u;
    3.0 * inv * inv * u * p1 + 3.0 * inv * u * u * p2 + u * u * u
}

fn bezier_component_derivative(u: f32, p1: f32, p2: f32) -> f32 {
    let inv = 1.0 - u;
    3.0 * inv * inv * p1 + 6.0 * inv * u * (p2 - p1) + 3.0 * u * u * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::{cubic_bezier_ease, ms_to_frames};
    use fret_ui::theme::CubicBezier;

    #[test]
    fn ms_to_frames_rounds_up() {
        assert_eq!(ms_to_frames(0), 0);
        assert_eq!(ms_to_frames(1), 1);
        assert_eq!(ms_to_frames(16), 1);
        assert_eq!(ms_to_frames(17), 2);
        assert_eq!(ms_to_frames(100), 6);
    }

    #[test]
    fn cubic_bezier_ease_is_monotonic_for_standard_curve() {
        let standard = CubicBezier {
            x1: 0.2,
            y1: 0.0,
            x2: 0.0,
            y2: 1.0,
        };
        let mut prev = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let y = cubic_bezier_ease(standard, t);
            assert!(y >= prev - 1e-3, "non-monotonic at t={t} y={y} prev={prev}");
            prev = y;
        }
    }
}
