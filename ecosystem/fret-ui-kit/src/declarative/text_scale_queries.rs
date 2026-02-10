use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns the best-effort text scale factor for the current window (ADR 1185).
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn text_scale_factor<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: f32,
) -> f32 {
    cx.environment_text_scale_factor(invalidation)
        .unwrap_or(default_when_unknown)
}
