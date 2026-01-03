//! Presence (Radix-aligned outcomes).
//!
//! Radix's Presence primitive is used to coordinate mount/unmount transitions while keeping
//! behavioral outcomes consistent (e.g. closing animations that remain paintable but not
//! interactive).
//!
//! Fret splits this into:
//!
//! - `crate::headless::presence`: deterministic state machine (`FadePresence`).
//! - `crate::declarative::presence`: runtime-driven driver (frame clock + redraw scheduling).
//!
//! This module provides a stable, Radix-named facade surface and keeps call sites from reaching
//! into the `declarative` module directly. See <https://github.com/radix-ui/primitives>.

use fret_ui::{ElementContext, UiHost};

pub use crate::headless::presence::{FadePresence, PresenceOutput};

/// Drive a fade presence transition using the UI runtime's monotonic frame clock.
///
/// This is a thin facade around `crate::declarative::presence::fade_presence`.
pub fn fade_presence<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    fade_ticks: u64,
) -> PresenceOutput {
    crate::declarative::presence::fade_presence(cx, open, fade_ticks)
}
