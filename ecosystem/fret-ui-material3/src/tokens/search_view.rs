//! Typed token access for Material 3 search views.
//!
//! Reference: Material Web v30 `md.comp.search-view.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn container_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-high"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.surface-container-high")
        })
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.search-view.container.elevation")
        .unwrap_or(Px(6.0))
}

pub(crate) fn divider_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.divider.color")
        .or_else(|| theme.color_by_key("md.sys.color.outline"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.outline"))
}

pub(crate) fn docked_container_shape(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.search-view.docked.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or(Px(28.0));
    Corners::all(r)
}

pub(crate) fn header_leading_icon_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.header.leading-icon.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn header_trailing_icon_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.header.trailing-icon.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
        })
}

pub(crate) fn header_input_text_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.header.input-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn header_supporting_text_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-view.header.supporting-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
        })
}

pub(crate) fn header_input_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key("md.comp.search-view.header.input-text")
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
        .unwrap_or_default()
}
