use fret_core::Edges;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns runner-committed safe-area insets for the current window (ADR 0232).
///
/// This is intended for future mobile targets. When unavailable, returns `None`.
#[track_caller]
pub fn safe_area_insets<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Option<Edges> {
    cx.environment_safe_area_insets(invalidation)
}

/// Returns safe-area insets for the current window, defaulting to zero when unavailable.
#[track_caller]
pub fn safe_area_insets_or_zero<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Edges {
    safe_area_insets(cx, invalidation).unwrap_or_default()
}
