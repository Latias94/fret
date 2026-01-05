//! ScrollArea primitives (Radix-aligned outcomes).
//!
//! This module provides a stable, Radix-named surface for composing scroll-area behavior in
//! recipes. It intentionally models outcomes rather than React/DOM APIs.
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/scroll-area/src/scroll-area.tsx`

/// Matches Radix ScrollArea `type` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollAreaType {
    Auto,
    Always,
    Scroll,
    Hover,
}

impl Default for ScrollAreaType {
    fn default() -> Self {
        // Radix default is `type="hover"`.
        Self::Hover
    }
}

/// Best-effort mapping from Radix scroll-area types to Fret's current scrollbar visibility model.
///
/// Fret currently supports:
/// - always show scrollbar, or
/// - only show scrollbar while hovered.
///
/// We do not yet model:
/// - `auto` (only show when overflowing), or
/// - `scroll` (show while scrolling + hide after delay).
pub fn show_scrollbar_for_hover_state(ty: ScrollAreaType, hovered: bool) -> bool {
    match ty {
        ScrollAreaType::Always => true,
        ScrollAreaType::Hover => hovered,
        // Best-effort fallbacks:
        ScrollAreaType::Auto => true,
        ScrollAreaType::Scroll => hovered,
    }
}

