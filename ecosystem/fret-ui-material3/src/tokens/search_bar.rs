//! Typed token access for Material 3 search bars.
//!
//! Reference: Material Web v30 `md.comp.search-bar.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.search-bar.container.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.search-bar.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0));
    Corners::all(r)
}

pub(crate) fn container_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-high"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.surface-container-high")
        })
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.search-bar.container.elevation")
        .unwrap_or(Px(6.0))
}

pub(crate) fn leading_icon_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.leading-icon.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn trailing_icon_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.trailing-icon.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
        })
}

pub(crate) fn input_text_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.input-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn supporting_text_color(theme: &Theme, hovered: bool, pressed: bool) -> Color {
    let key = if pressed {
        "md.comp.search-bar.pressed.supporting-text.color"
    } else if hovered {
        "md.comp.search-bar.hover.supporting-text.color"
    } else {
        "md.comp.search-bar.supporting-text.color"
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| {
            MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface-variant")
        })
}

pub(crate) fn input_text_style(theme: &Theme) -> TextStyle {
    theme
        .text_style_by_key("md.comp.search-bar.input-text")
        .or_else(|| theme.text_style_by_key("md.sys.typescale.body-large"))
        .unwrap_or_default()
}

pub(crate) fn hover_state_layer_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.hover.state-layer.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn pressed_state_layer_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.search-bar.pressed.state-layer.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
}

pub(crate) fn hover_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.search-bar.hover.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
        .unwrap_or(0.08)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.search-bar.pressed.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}
