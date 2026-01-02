//! Hover intent (Radix-aligned outcomes).
//!
//! Radix components like Tooltip and HoverCard rely on a small "hover intent" policy:
//! delay-open and/or delay-close based on pointer hover, with deterministic transitions.
//!
//! In Fret:
//!
//! - `crate::headless::hover_intent` provides the deterministic state machine.
//! - `crate::declarative::hover_intent` drives it from the runtime clock + schedules frames.
//!
//! This module provides a stable, Radix-aligned entry point and keeps component call sites from
//! reaching into `crate::declarative` directly.

use fret_ui::{ElementContext, UiHost};

pub use crate::headless::hover_intent::{HoverIntentConfig, HoverIntentState, HoverIntentUpdate};

/// Drive hover intent using the UI runtime clock (frame count) and request continuous frames while
/// delayed transitions are pending.
pub fn drive<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    hovered: bool,
    cfg: HoverIntentConfig,
) -> HoverIntentUpdate {
    crate::declarative::hover_intent::hover_intent(cx, hovered, cfg)
}
