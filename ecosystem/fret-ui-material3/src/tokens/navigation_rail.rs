//! Typed token access for Material 3 navigation rails.
//!
//! This module centralizes token key mapping and fallback chains so navigation rail outcomes
//! remain stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::tokens::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationRailItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.container.width")
        .unwrap_or(Px(80.0))
}

pub(crate) fn item_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.item.width")
        .unwrap_or_else(|| container_width(theme))
}

pub(crate) fn item_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.item.height")
        .or_else(|| theme.metric_by_key("md.comp.navigation-rail.active-indicator.width"))
        .unwrap_or(Px(56.0))
}

pub(crate) fn vertical_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.vertical-padding")
        .unwrap_or(Px(4.0))
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-rail.container.color",
        "md.sys.color.surface",
    )
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    let radius = theme
        .metric_by_key("md.comp.navigation-rail.container.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.none"))
        .unwrap_or(Px(0.0));
    Corners::all(radius)
}

pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.active-indicator.width")
        .unwrap_or(Px(56.0))
}

pub(crate) fn active_indicator_height(theme: &Theme, has_label: bool) -> Px {
    if has_label {
        theme
            .metric_by_key("md.comp.navigation-rail.active-indicator.height")
            .unwrap_or(Px(32.0))
    } else {
        theme
            .metric_by_key("md.comp.navigation-rail.no-label.active-indicator.height")
            .unwrap_or(Px(56.0))
    }
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-rail.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn active_indicator_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-rail.active-indicator.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(active_indicator_radius(theme))
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    state_layer_opacity(theme, NavigationRailItemInteraction::Pressed)
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    interaction: NavigationRailItemInteraction,
) -> f32 {
    let Some((key, interaction)) = state_layer_opacity_token(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(key, interaction)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationRailItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }
    state_layer_opacity(theme, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationRailItemInteraction,
) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        state_layer_color_key(active, interaction),
        "md.sys.color.on-surface",
    )
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationRailItemInteraction,
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
    interaction: NavigationRailItemInteraction,
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
        "md.comp.navigation-rail.active.label-text.weight"
    } else {
        "md.comp.navigation-rail.label-text.weight"
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
        .metric_by_key("md.comp.navigation-rail.icon.size")
        .unwrap_or(Px(24.0))
}

fn state_layer_color_key(active: bool, interaction: NavigationRailItemInteraction) -> &'static str {
    match (active, interaction) {
        (_, NavigationRailItemInteraction::Default) => {
            "md.comp.navigation-rail.inactive.hover.state-layer.color"
        }
        (true, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.active.focus.state-layer.color"
        }
        (true, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.active.hover.state-layer.color"
        }
        (true, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.active.pressed.state-layer.color"
        }
        (false, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.inactive.focus.state-layer.color"
        }
        (false, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.inactive.hover.state-layer.color"
        }
        (false, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.inactive.pressed.state-layer.color"
        }
    }
}

fn state_layer_opacity_token(
    interaction: NavigationRailItemInteraction,
) -> Option<(&'static str, MaterialStateLayerInteraction)> {
    match interaction {
        NavigationRailItemInteraction::Default => None,
        NavigationRailItemInteraction::Pressed => Some((
            "md.comp.navigation-rail.pressed.state-layer.opacity",
            MaterialStateLayerInteraction::Pressed,
        )),
        NavigationRailItemInteraction::Focused => Some((
            "md.comp.navigation-rail.focus.state-layer.opacity",
            MaterialStateLayerInteraction::Focused,
        )),
        NavigationRailItemInteraction::Hovered => Some((
            "md.comp.navigation-rail.hover.state-layer.opacity",
            MaterialStateLayerInteraction::Hovered,
        )),
    }
}

fn icon_color_key(active: bool, interaction: NavigationRailItemInteraction) -> &'static str {
    match (active, interaction) {
        (true, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.active.focus.icon.color"
        }
        (true, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.active.hover.icon.color"
        }
        (true, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.active.pressed.icon.color"
        }
        (true, NavigationRailItemInteraction::Default) => {
            "md.comp.navigation-rail.active.icon.color"
        }
        (false, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.inactive.focus.icon.color"
        }
        (false, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.inactive.hover.icon.color"
        }
        (false, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.inactive.pressed.icon.color"
        }
        (false, NavigationRailItemInteraction::Default) => {
            "md.comp.navigation-rail.inactive.icon.color"
        }
    }
}

fn label_color_key(active: bool, interaction: NavigationRailItemInteraction) -> &'static str {
    match (active, interaction) {
        (true, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.active.focus.label-text.color"
        }
        (true, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.active.hover.label-text.color"
        }
        (true, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.active.pressed.label-text.color"
        }
        (true, NavigationRailItemInteraction::Default) => {
            "md.comp.navigation-rail.active.label-text.color"
        }
        (false, NavigationRailItemInteraction::Focused) => {
            "md.comp.navigation-rail.inactive.focus.label-text.color"
        }
        (false, NavigationRailItemInteraction::Hovered) => {
            "md.comp.navigation-rail.inactive.hover.label-text.color"
        }
        (false, NavigationRailItemInteraction::Pressed) => {
            "md.comp.navigation-rail.inactive.pressed.label-text.color"
        }
        (false, NavigationRailItemInteraction::Default) => {
            "md.comp.navigation-rail.inactive.label-text.color"
        }
    }
}
