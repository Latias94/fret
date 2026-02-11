use fret_core::Edges;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns runner-committed viewport occlusion insets for the current window (ADR 0232).
///
/// These insets are intended for transient obstructions like virtual keyboards (IME) on mobile.
/// When unavailable, returns `None`.
#[track_caller]
pub fn occlusion_insets<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Option<Edges> {
    cx.environment_occlusion_insets(invalidation)
}

/// Returns viewport occlusion insets for the current window, defaulting to zero when unavailable.
#[track_caller]
pub fn occlusion_insets_or_zero<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Edges {
    occlusion_insets(cx, invalidation).unwrap_or_default()
}
