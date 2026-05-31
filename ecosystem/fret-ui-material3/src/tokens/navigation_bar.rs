//! Typed token access for Material 3 navigation bars.
//!
//! This module centralizes token key mapping and fallback chains so navigation bar outcomes remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationBarItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.container.height")
        .unwrap_or(Px(80.0))
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.container.color",
        "md.sys.color.surface-container",
    )
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.container.elevation")
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.container.shadow-color",
        "md.sys.color.shadow",
    )
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let radius = theme
        .metric_by_key("md.comp.navigation-bar.container.shape")
        .unwrap_or(Px(0.0));
    Corners::all(radius)
}

pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.active-indicator.width")
        .unwrap_or(Px(64.0))
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.active-indicator.height")
        .unwrap_or(Px(32.0))
}

pub(crate) fn active_indicator_top_offset(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.active-indicator.top-offset")
        .unwrap_or(Px(12.0))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-bar.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn active_indicator_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.active-indicator.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(active_indicator_radius(theme))
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    state_layer_opacity(theme, NavigationBarItemInteraction::Pressed)
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: NavigationBarItemInteraction) -> f32 {
    let Some((key, interaction)) = state_layer_opacity_token(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(key, interaction)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationBarItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }
    state_layer_opacity(theme, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    let key = state_layer_color_key(active, interaction);
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface")
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    let fallback = if active {
        "md.sys.color.on-secondary-container"
    } else {
        "md.sys.color.on-surface-variant"
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(icon_color_key(active, interaction), fallback)
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    let fallback = if active {
        "md.sys.color.on-surface"
    } else {
        "md.sys.color.on-surface-variant"
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(label_color_key(active, interaction), fallback)
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    let weight_key = if active {
        "md.comp.navigation-bar.active.label-text.weight"
    } else {
        "md.comp.navigation-bar.label-text.weight"
    };
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-medium",
        weight_key,
        if active { 700.0 } else { 500.0 },
        TextIntent::Control,
    )
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.icon.size")
        .unwrap_or(Px(24.0))
}

pub(crate) fn item_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.item.gap")
        .unwrap_or(Px(8.0))
}

fn state_layer_color_key(active: bool, interaction: NavigationBarItemInteraction) -> &'static str {
    match (active, interaction) {
        (_, NavigationBarItemInteraction::Default) => {
            "md.comp.navigation-bar.inactive.hover.state-layer.color"
        }
        (true, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.active.focus.state-layer.color"
        }
        (true, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.active.hover.state-layer.color"
        }
        (true, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.active.pressed.state-layer.color"
        }
        (false, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.inactive.focus.state-layer.color"
        }
        (false, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.inactive.hover.state-layer.color"
        }
        (false, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.inactive.pressed.state-layer.color"
        }
    }
}

fn state_layer_opacity_token(
    interaction: NavigationBarItemInteraction,
) -> Option<(&'static str, MaterialStateLayerInteraction)> {
    match interaction {
        NavigationBarItemInteraction::Default => None,
        NavigationBarItemInteraction::Pressed => Some((
            "md.comp.navigation-bar.pressed.state-layer.opacity",
            MaterialStateLayerInteraction::Pressed,
        )),
        NavigationBarItemInteraction::Focused => Some((
            "md.comp.navigation-bar.focus.state-layer.opacity",
            MaterialStateLayerInteraction::Focused,
        )),
        NavigationBarItemInteraction::Hovered => Some((
            "md.comp.navigation-bar.hover.state-layer.opacity",
            MaterialStateLayerInteraction::Hovered,
        )),
    }
}

fn icon_color_key(active: bool, interaction: NavigationBarItemInteraction) -> &'static str {
    match (active, interaction) {
        (true, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.active.focus.icon.color"
        }
        (true, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.active.hover.icon.color"
        }
        (true, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.active.pressed.icon.color"
        }
        (true, NavigationBarItemInteraction::Default) => "md.comp.navigation-bar.active.icon.color",
        (false, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.inactive.focus.icon.color"
        }
        (false, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.inactive.hover.icon.color"
        }
        (false, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.inactive.pressed.icon.color"
        }
        (false, NavigationBarItemInteraction::Default) => {
            "md.comp.navigation-bar.inactive.icon.color"
        }
    }
}

fn label_color_key(active: bool, interaction: NavigationBarItemInteraction) -> &'static str {
    match (active, interaction) {
        (true, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.active.focus.label-text.color"
        }
        (true, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.active.hover.label-text.color"
        }
        (true, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.active.pressed.label-text.color"
        }
        (true, NavigationBarItemInteraction::Default) => {
            "md.comp.navigation-bar.active.label-text.color"
        }
        (false, NavigationBarItemInteraction::Focused) => {
            "md.comp.navigation-bar.inactive.focus.label-text.color"
        }
        (false, NavigationBarItemInteraction::Hovered) => {
            "md.comp.navigation-bar.inactive.hover.label-text.color"
        }
        (false, NavigationBarItemInteraction::Pressed) => {
            "md.comp.navigation-bar.inactive.pressed.label-text.color"
        }
        (false, NavigationBarItemInteraction::Default) => {
            "md.comp.navigation-bar.inactive.label-text.color"
        }
    }
}
