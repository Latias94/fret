//! Typed token access for Material 3 navigation drawers.
//!
//! This module centralizes token key mapping and fallback chains so navigation drawer outcomes
//! remain stable and drift-resistant during refactors.

use fret_core::{Color, Corners, FontWeight, Px};
use fret_ui::Theme;

use crate::navigation_drawer::NavigationDrawerVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavigationDrawerItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.container.width")
        .unwrap_or(Px(360.0))
}

pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.active-indicator.width")
        .unwrap_or(Px(336.0))
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    let container_w = container_width(theme);
    let active_w = active_indicator_width(theme);
    Px(((container_w.0 - active_w.0) / 2.0).max(0.0))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.navigation-drawer.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or_else(|| Corners::all(Px(0.0)))
}

pub(crate) fn container_background(theme: &Theme, variant: NavigationDrawerVariant) -> Color {
    let (key, fallback) = match variant {
        NavigationDrawerVariant::Standard => (
            "md.comp.navigation-drawer.standard.container.color",
            "md.sys.color.surface",
        ),
        NavigationDrawerVariant::Modal => (
            "md.comp.navigation-drawer.modal.container.color",
            "md.sys.color.surface-container-low",
        ),
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| theme.color_required(fallback))
}

pub(crate) fn container_elevation(theme: &Theme, variant: NavigationDrawerVariant) -> Px {
    match variant {
        NavigationDrawerVariant::Standard => theme
            .metric_by_key("md.comp.navigation-drawer.standard.container.elevation")
            .unwrap_or(Px(0.0)),
        NavigationDrawerVariant::Modal => theme
            .metric_by_key("md.comp.navigation-drawer.modal.container.elevation")
            .unwrap_or(Px(1.0)),
    }
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.active-indicator.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn active_indicator_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.active-indicator.shape")
        .or_else(|| theme.metric_by_key("md.sys.shape.corner.full"))
        .unwrap_or(Px(9999.0))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.navigation-drawer.active-indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.secondary-container"))
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.navigation-drawer.pressed.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationDrawerItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    match interaction {
        NavigationDrawerItemInteraction::Default => 0.0,
        NavigationDrawerItemInteraction::Pressed => theme
            .number_by_key("md.comp.navigation-drawer.pressed.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationDrawerItemInteraction::Focused => theme
            .number_by_key("md.comp.navigation-drawer.focus.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        NavigationDrawerItemInteraction::Hovered => theme
            .number_by_key("md.comp.navigation-drawer.hover.state-layer.opacity")
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
}

fn label_color_key(active: bool, interaction: NavigationDrawerItemInteraction) -> &'static str {
    if active {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.active.focus.label-text.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.active.hover.label-text.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.active.pressed.label-text.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.active.label-text.color"
            }
        }
    } else {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.inactive.focus.label-text.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.inactive.hover.label-text.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.inactive.pressed.label-text.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.inactive.label-text.color"
            }
        }
    }
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    let fallback = if active {
        "md.sys.color.on-secondary-container"
    } else {
        "md.sys.color.on-surface-variant"
    };
    theme
        .color_by_key(label_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| theme.color_required(fallback))
}

fn icon_color_key(active: bool, interaction: NavigationDrawerItemInteraction) -> &'static str {
    if active {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.active.focus.icon.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.active.hover.icon.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.active.pressed.icon.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.active.icon.color"
            }
        }
    } else {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.inactive.focus.icon.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.inactive.hover.icon.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.inactive.pressed.icon.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.inactive.icon.color"
            }
        }
    }
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    let fallback = if active {
        "md.sys.color.on-secondary-container"
    } else {
        "md.sys.color.on-surface-variant"
    };
    theme
        .color_by_key(icon_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| theme.color_required(fallback))
}

fn state_layer_color_key(
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> &'static str {
    if active {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.active.focus.state-layer.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.active.hover.state-layer.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.active.pressed.state-layer.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.active.focus.state-layer.color"
            }
        }
    } else {
        match interaction {
            NavigationDrawerItemInteraction::Focused => {
                "md.comp.navigation-drawer.inactive.focus.state-layer.color"
            }
            NavigationDrawerItemInteraction::Hovered => {
                "md.comp.navigation-drawer.inactive.hover.state-layer.color"
            }
            NavigationDrawerItemInteraction::Pressed => {
                "md.comp.navigation-drawer.inactive.pressed.state-layer.color"
            }
            NavigationDrawerItemInteraction::Default => {
                "md.comp.navigation-drawer.inactive.hover.state-layer.color"
            }
        }
    }
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    let fallback = if active {
        "md.sys.color.on-secondary-container"
    } else {
        "md.sys.color.on-surface"
    };
    theme
        .color_by_key(state_layer_color_key(active, interaction))
        .or_else(|| theme.color_by_key(fallback))
        .unwrap_or_else(|| theme.color_required(fallback))
}

pub(crate) fn label_weight(theme: &Theme, active: bool) -> FontWeight {
    let weight = if active {
        theme
            .number_by_key("md.comp.navigation-drawer.active.label-text.weight")
            .unwrap_or(700.0)
    } else {
        theme
            .number_by_key("md.comp.navigation-drawer.label-text.weight")
            .unwrap_or(500.0)
    };
    FontWeight(weight.round().clamp(1.0, 1000.0) as u16)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.icon.size")
        .unwrap_or(Px(24.0))
}
