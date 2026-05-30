//! Typed token access for Material 3 badges.
//!
//! Reference: Material Web v30 `md.comp.badge.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::typography;

pub(crate) fn dot_size(theme: &Theme) -> Px {
    theme.metric_by_key("md.comp.badge.size").unwrap_or(Px(6.0))
}

pub(crate) fn large_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.badge.large.size")
        .unwrap_or(Px(16.0))
}

pub(crate) fn dot_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.badge.color")
        .or_else(|| theme.color_by_key("md.sys.color.error"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.error"))
}

pub(crate) fn large_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.badge.large.color")
        .or_else(|| theme.color_by_key("md.sys.color.error"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.error"))
}

pub(crate) fn large_label_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.badge.large.label-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-error"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-error"))
}

pub(crate) fn large_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight(
        theme,
        Some("md.comp.badge.large.label-text"),
        "md.sys.typescale.label-small",
        Some("md.comp.badge.large.label-text.weight"),
        TextIntent::Control,
    )
}

pub(crate) fn shape(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.badge.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0));
    Corners::all(r)
}

pub(crate) fn large_shape(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.badge.large.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0));
    Corners::all(r)
}
