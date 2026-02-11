use fret_core::ColorScheme;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns a policy-friendly color scheme value.
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn preferred_color_scheme<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: ColorScheme,
) -> ColorScheme {
    cx.environment_color_scheme(invalidation)
        .unwrap_or(default_when_unknown)
}

/// Returns a policy-friendly boolean for "dark mode".
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn prefers_dark_color_scheme<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    match cx.environment_color_scheme(invalidation) {
        Some(ColorScheme::Dark) => true,
        Some(ColorScheme::Light) => false,
        None => default_when_unknown,
    }
}
