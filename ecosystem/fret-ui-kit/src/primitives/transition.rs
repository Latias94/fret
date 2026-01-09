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

use fret_ui::{ElementContext, UiHost};

pub use crate::headless::transition::{TransitionOutput, TransitionTimeline};

/// Drive a transition using the UI runtime's monotonic clock (same duration for open/close).
pub fn drive_transition<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    ticks: u64,
) -> TransitionOutput {
    crate::declarative::transition::drive_transition(cx, open, ticks)
}

/// Drive a transition using the UI runtime's monotonic clock, with separate open/close durations.
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
