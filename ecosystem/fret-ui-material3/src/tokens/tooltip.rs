//! Typed token access for Material 3 tooltips.
//!
//! This module centralizes token key mapping and fallback chains so tooltip outcomes remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Edges, Px};
use fret_ui::Theme;

pub(crate) fn plain_container_background(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.plain-tooltip.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.inverse-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.inverse-surface"))
}

pub(crate) fn plain_supporting_text_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.plain-tooltip.supporting-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.inverse-on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.inverse-on-surface"))
}

pub(crate) fn plain_container_shape_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.plain-tooltip.container.shape")
        .unwrap_or(Px(4.0))
}

pub(crate) fn plain_container_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges {
        left: Px(8.0),
        right: Px(8.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
}

pub(crate) fn max_width(theme: &Theme) -> Px {
    let _ = theme;
    Px(240.0)
}

pub(crate) fn shadow_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.sys.color.shadow")
        .unwrap_or_else(|| theme.color_required("md.sys.color.shadow"))
}

pub(crate) fn close_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.short1")
        .unwrap_or(50)
}
