use fret_core::ForcedColorsMode;
use fret_ui::{ElementContext, Invalidation, UiHost};

/// Returns a policy-friendly forced-colors mode.
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn forced_colors_mode<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: ForcedColorsMode,
) -> ForcedColorsMode {
    cx.environment_forced_colors_mode(invalidation)
        .unwrap_or(default_when_unknown)
}

/// Returns a policy-friendly boolean for "forced colors active".
///
/// When the environment value is unknown/unavailable, `default_when_unknown` is returned.
#[track_caller]
pub fn forced_colors_active<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    invalidation: Invalidation,
    default_when_unknown: bool,
) -> bool {
    match cx.environment_forced_colors_mode(invalidation) {
        Some(ForcedColorsMode::Active) => true,
        Some(ForcedColorsMode::None) => false,
        None => default_when_unknown,
    }
}
