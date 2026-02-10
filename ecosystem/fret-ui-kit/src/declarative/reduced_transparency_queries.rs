use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns a policy-friendly reduced-transparency boolean (ADR 1185).
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn prefers_reduced_transparency<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    cx.environment_prefers_reduced_transparency(invalidation)
        .unwrap_or(default_when_unknown)
}
