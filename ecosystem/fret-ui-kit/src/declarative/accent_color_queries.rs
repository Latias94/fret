use fret_core::Color;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns the best-effort system accent color for the current window (ADR 1185).
#[track_caller]
pub fn accent_color<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
) -> Option<Color> {
    cx.environment_accent_color(invalidation)
}
