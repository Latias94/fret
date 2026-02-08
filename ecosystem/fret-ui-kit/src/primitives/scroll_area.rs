use fret_ui::scroll::ScrollHandle;
use fret_ui::{ElementContext, UiHost};

pub use fret_ui_headless::scroll_area::ScrollAreaType;
pub use fret_ui_headless::scroll_area_visibility::ScrollAreaVisibilityOutput;

pub use crate::declarative::scroll_area_visibility::{
    DEFAULT_SCROLL_END_DEBOUNCE_TICKS, DEFAULT_SCROLL_HIDE_DELAY_TICKS,
};

/// Compute scrollbar visibility for Radix-aligned `type` modes and drive time-based transitions.
///
/// This facade:
/// - measures overflow using the imperative `ScrollHandle`,
/// - detects "scrolling" by comparing handle offsets across frames,
/// - and schedules redraws while time-based visibility transitions are pending.
pub fn scrollbar_visibility<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    ty: ScrollAreaType,
    hovered: bool,
    handle: ScrollHandle,
    scroll_hide_delay_ticks: u64,
) -> ScrollAreaVisibilityOutput {
    crate::declarative::scroll_area_visibility::scrollbar_visibility(
        cx,
        ty,
        hovered,
        handle,
        scroll_hide_delay_ticks,
    )
}

/// Legacy best-effort mapping from Radix scroll-area types to a hover-gated boolean.
///
/// Prefer [`scrollbar_visibility`] when you need Radix-aligned `auto`, `scroll`, and delayed hide.
pub fn show_scrollbar_for_hover_state(ty: ScrollAreaType, hovered: bool) -> bool {
    match ty {
        ScrollAreaType::Always => true,
        ScrollAreaType::Hover => hovered,
        // Best-effort fallbacks:
        ScrollAreaType::Auto => true,
        ScrollAreaType::Scroll => hovered,
    }
}
