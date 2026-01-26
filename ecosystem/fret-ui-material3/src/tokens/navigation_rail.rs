//! Typed token access for Material 3 navigation rails.
//!
//! This module centralizes token key mapping and fallback chains so navigation rail outcomes
//! remain stable and drift-resistant during refactors.

use fret_core::{Color, Corners, FontWeight, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

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

pub(crate) fn active_indicator_height(theme: &Theme, always_show_label: bool) -> Px {
    if always_show_label {
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
    match interaction {
        NavigationRailItemInteraction::Default => 0.0,
        NavigationRailItemInteraction::Pressed => theme
            .number_by_key("md.comp.navigation-rail.pressed.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationRailItemInteraction::Focused => theme
            .number_by_key("md.comp.navigation-rail.focus.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationRailItemInteraction::Hovered => theme
            .number_by_key("md.comp.navigation-rail.hover.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
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
    theme
        .color_by_key(state_layer_color_key(active, interaction))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
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
    theme
        .color_by_key(icon_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys(fallback))
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
    theme
        .color_by_key(label_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys(fallback))
}

pub(crate) fn label_weight(theme: &Theme, active: bool) -> FontWeight {
    let weight = if active {
        theme
            .number_by_key("md.comp.navigation-rail.active.label-text.weight")
            .unwrap_or(700.0)
    } else {
        theme
            .number_by_key("md.comp.navigation-rail.label-text.weight")
            .unwrap_or(500.0)
    };
    FontWeight(weight.round().clamp(1.0, 1000.0) as u16)
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
