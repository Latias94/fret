//! Typed token access for Material 3 navigation bars.
//!
//! This module centralizes token key mapping and fallback chains so navigation bar outcomes remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, FontWeight, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

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
    match interaction {
        NavigationBarItemInteraction::Default => 0.0,
        NavigationBarItemInteraction::Pressed => theme
            .number_by_key("md.comp.navigation-bar.pressed.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationBarItemInteraction::Focused => theme
            .number_by_key("md.comp.navigation-bar.focus.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationBarItemInteraction::Hovered => theme
            .number_by_key("md.comp.navigation-bar.hover.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
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
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys("md.sys.color.on-surface"))
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
    theme
        .color_by_key(icon_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys(fallback))
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
    theme
        .color_by_key(label_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| MaterialTokenResolver::new(theme).color_sys(fallback))
}

pub(crate) fn label_weight(theme: &Theme, active: bool) -> FontWeight {
    let weight = if active {
        theme
            .number_by_key("md.comp.navigation-bar.active.label-text.weight")
            .unwrap_or(700.0)
    } else {
        theme
            .number_by_key("md.comp.navigation-bar.label-text.weight")
            .unwrap_or(500.0)
    };
    FontWeight(weight.round().clamp(1.0, 1000.0) as u16)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-bar.icon.size")
        .unwrap_or(Px(24.0))
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
