//! Typed token access for Material 3 progress indicators.
//!
//! Material Web v30 ships a merged token set:
//! - shared colors/shapes: `md.comp.progress-indicator.*`
//! - linear metrics: `md.comp.progress-indicator.linear.*`
//! - circular metrics: `md.comp.progress-indicator.circular.*`
//!
//! This module centralizes token key mapping and fallback chains so progress indicator visuals
//! remain stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn track_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.progress-indicator.track.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn active_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.progress-indicator.active-indicator.color",
        "md.sys.color.primary",
    )
}

pub(crate) fn four_color_palette(theme: &Theme) -> [Color; 4] {
    // Align with Material Web v30 deprecated four-color progress indicator token defaults:
    // - one:   md.sys.color.primary
    // - two:   md.sys.color.primary-container
    // - three: md.sys.color.tertiary
    // - four:  md.sys.color.tertiary-container
    let tokens = MaterialTokenResolver::new(theme);
    [
        tokens.color_sys("md.sys.color.primary"),
        tokens.color_sys("md.sys.color.primary-container"),
        tokens.color_sys("md.sys.color.tertiary"),
        tokens.color_sys("md.sys.color.tertiary-container"),
    ]
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.progress-indicator.track.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .map(Corners::all)
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn active_shape(theme: &Theme) -> Corners {
    theme
        .metric_by_key("md.comp.progress-indicator.active-indicator.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .map(Corners::all)
        .unwrap_or_else(|| Corners::all(Px(9999.0)))
}

pub(crate) fn linear_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.linear.height")
        .unwrap_or(Px(4.0))
}

pub(crate) fn linear_track_thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.linear.track.thickness")
        .or_else(|| theme.metric_by_key("md.comp.progress-indicator.track.thickness"))
        .unwrap_or(Px(4.0))
}

pub(crate) fn linear_active_thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.linear.active-indicator.thickness")
        .or_else(|| theme.metric_by_key("md.comp.progress-indicator.active-indicator.thickness"))
        .unwrap_or(Px(4.0))
}

pub(crate) fn circular_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.circular.size")
        .unwrap_or(Px(40.0))
}

pub(crate) fn circular_track_thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.circular.track.thickness")
        .or_else(|| theme.metric_by_key("md.comp.progress-indicator.track.thickness"))
        .unwrap_or(Px(4.0))
}

pub(crate) fn circular_active_thickness(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.progress-indicator.circular.active-indicator.thickness")
        .or_else(|| theme.metric_by_key("md.comp.progress-indicator.active-indicator.thickness"))
        .unwrap_or(Px(4.0))
}
