//! Typed token access for Material 3 search bars.
//!
//! Reference: Material Web v30 `md.comp.search-bar.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul};
use crate::tokens::typography;

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.search-bar.container.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn container_min_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.sys.fret.material.search-bar.container.min-width")
        .unwrap_or(Px(360.0))
}

pub(crate) fn container_max_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.sys.fret.material.search-bar.container.max-width")
        .unwrap_or(Px(720.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let r = theme
        .metric_by_key("md.comp.search-bar.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0));
    Corners::all(r)
}

pub(crate) fn container_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.container.color",
        "md.sys.color.surface-container-high",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.search-bar.container.elevation")
        .unwrap_or(Px(6.0))
}

pub(crate) fn leading_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.leading-icon.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn trailing_icon_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.trailing-icon.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn input_text_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.input-text.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn supporting_text_color(theme: &Theme, hovered: bool, pressed: bool) -> Color {
    let key = if pressed {
        "md.comp.search-bar.pressed.supporting-text.color"
    } else if hovered {
        "md.comp.search-bar.hover.supporting-text.color"
    } else {
        "md.comp.search-bar.supporting-text.color"
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface-variant")
}

pub(crate) fn input_text_style(theme: &Theme) -> TextStyle {
    typography::text_style(
        theme,
        Some("md.comp.search-bar.input-text"),
        "md.sys.typescale.body-large",
        TextIntent::Control,
    )
}

pub(crate) fn hover_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.hover.state-layer.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn pressed_state_layer_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.search-bar.pressed.state-layer.color",
        "md.sys.color.on-surface",
    )
}

pub(crate) fn hover_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_comp_or_sys(
        "md.comp.search-bar.hover.state-layer.opacity",
        "md.sys.state.hover.state-layer-opacity",
        0.08,
    )
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).number_comp_or_sys(
        "md.comp.search-bar.pressed.state-layer.opacity",
        "md.sys.state.pressed.state-layer-opacity",
        0.1,
    )
}

pub(crate) fn selection_color(theme: &Theme) -> Color {
    let primary = MaterialTokenResolver::new(theme).color_sys("md.sys.color.primary");
    alpha_mul(primary, 0.35)
}

pub(crate) fn caret_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_sys("md.sys.color.primary")
}
