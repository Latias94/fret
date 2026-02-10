use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns a policy-friendly reduced-motion boolean.
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn prefers_reduced_motion<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    cx.environment_prefers_reduced_motion(invalidation)
        .unwrap_or(default_when_unknown)
}
