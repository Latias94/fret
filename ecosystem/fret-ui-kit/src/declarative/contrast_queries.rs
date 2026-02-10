use fret_core::ContrastPreference;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns a policy-friendly contrast preference.
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn contrast_preference<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: ContrastPreference,
) -> ContrastPreference {
    cx.environment_prefers_contrast(invalidation)
        .unwrap_or(default_when_unknown)
}

/// Returns a policy-friendly boolean for "prefers more contrast".
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn prefers_more_contrast<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    match cx.environment_prefers_contrast(invalidation) {
        Some(ContrastPreference::More) => true,
        Some(ContrastPreference::Less)
        | Some(ContrastPreference::NoPreference)
        | Some(ContrastPreference::Custom) => false,
        None => default_when_unknown,
    }
}
