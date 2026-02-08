//! Transition timelines (deterministic, tick-driven).
//!
//! This module provides a small, stable facade around Fret's transition substrate:
//!
//! - a headless state machine: [`crate::headless::transition::TransitionTimeline`]
//! - a runtime-driven driver: [`crate::declarative::transition`]
//!
//! Radix does not ship a "Transition" primitive as a public package, but multiple Radix primitives
//! rely on the same underlying *presence-like* outcome:
//!
//! - keep content mounted while closing animations run
//! - expose a normalized progress value for mapping into opacity/transform/etc.
//!
//! In Fret, this is modeled as a generic `TransitionTimeline` plus a deterministic runtime driver
//! that:
//! - advances on a monotonic tick source (frame/app ticks),
//! - holds a continuous-frames lease while animating, and
//! - requests redraws while animating.
//!
//! Most overlay-ish components should prefer [`crate::primitives::presence`] (Radix `Presence`
//! outcome). This module is intended for non-overlay transitions (e.g. collapsible height motion)
//! or for authoring new presence-like drivers.

use fret_core::{Px, Size};
use fret_ui::{ElementContext, UiHost};

pub use crate::headless::transition::{TransitionOutput, TransitionTimeline};

/// Shared transition profile for binary open/close state machines.
///
/// This mirrors the common Base UI/Radix pattern where components keep a mounted/present state
/// while transitioning between two boolean states and expose a progress value for style mapping.
#[derive(Clone, Copy)]
pub struct TransitionProfile {
    pub open_ticks: u64,
    pub close_ticks: u64,
    pub ease: fn(f32) -> f32,
}

impl TransitionProfile {
    pub fn new(open_ticks: u64, close_ticks: u64, ease: fn(f32) -> f32) -> Self {
        Self {
            open_ticks,
            close_ticks,
            ease,
        }
    }
}

/// Drive a transition using the UI runtime's monotonic clock (same duration for open/close).
#[track_caller]
pub fn drive_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    ticks: u64,
) -> TransitionOutput {
    crate::declarative::transition::drive_transition(cx, open, ticks)
}

/// Drive a transition using the UI runtime's monotonic clock, with separate open/close durations.
#[track_caller]
pub fn drive_transition_with_durations<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
) -> TransitionOutput {
    crate::declarative::transition::drive_transition_with_durations(
        cx,
        open,
        open_ticks,
        close_ticks,
    )
}

/// Drive a transition with separate durations and a custom easing curve.
#[track_caller]
pub fn drive_transition_with_durations_and_easing<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    open_ticks: u64,
    close_ticks: u64,
    ease: fn(f32) -> f32,
) -> TransitionOutput {
    crate::declarative::transition::drive_transition_with_durations_and_easing(
        cx,
        open,
        open_ticks,
        close_ticks,
        ease,
    )
}

/// Drive a transition using a reusable [`TransitionProfile`].
#[track_caller]
pub fn drive_transition_with_profile<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    profile: TransitionProfile,
) -> TransitionOutput {
    drive_transition_with_durations_and_easing(
        cx,
        open,
        profile.open_ticks,
        profile.close_ticks,
        profile.ease,
    )
}

/// Linear interpolation for pixel values.
pub fn lerp_px(from: Px, to: Px, t: f32) -> Px {
    let t = t.clamp(0.0, 1.0);
    Px(from.0 + (to.0 - from.0) * t)
}

/// Linear interpolation for sizes.
pub fn lerp_size(from: Size, to: Size, t: f32) -> Size {
    Size::new(
        lerp_px(from.width, to.width, t),
        lerp_px(from.height, to.height, t),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size as CoreSize};
    use fret_runtime::{FrameId, TickId};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            CoreSize::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn lerp_px_is_clamped_and_monotonic() {
        assert_eq!(lerp_px(Px(0.0), Px(10.0), -1.0), Px(0.0));
        assert_eq!(lerp_px(Px(0.0), Px(10.0), 0.0), Px(0.0));
        assert_eq!(lerp_px(Px(0.0), Px(10.0), 0.5), Px(5.0));
        assert_eq!(lerp_px(Px(0.0), Px(10.0), 1.0), Px(10.0));
        assert_eq!(lerp_px(Px(0.0), Px(10.0), 2.0), Px(10.0));
    }

    #[test]
    fn wrapper_drivers_keep_independent_state_per_call_site() {
        let window = AppWindowId::default();
        let mut app = App::new();

        app.set_tick_id(TickId(1));
        app.set_frame_id(FrameId(1));

        let (a, b) = fret_ui::elements::with_element_cx(&mut app, window, bounds(), "t", |cx| {
            let a = drive_transition_with_durations(cx, true, 6, 6);
            let b = drive_transition_with_durations(cx, false, 6, 6);
            (a, b)
        });

        assert!(a.present);
        assert!(a.animating);
        assert!(a.progress > 0.0 && a.progress < 1.0);

        assert!(!b.present);
        assert!(!b.animating);
        assert_eq!(b.progress, 0.0);
    }
}
