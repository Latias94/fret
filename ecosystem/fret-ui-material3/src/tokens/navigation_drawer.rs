//! Typed token access for Material 3 navigation drawers.
//!
//! This module centralizes token key mapping and fallback chains so navigation drawer outcomes
//! remain stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::TextIntent;

use crate::foundation::token_resolver::{MaterialStateLayerInteraction, MaterialTokenResolver};
use crate::navigation_drawer::NavigationDrawerVariant;
use crate::tokens::typography;

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

    MaterialTokenResolver::new(theme).color_comp_or_sys(key, fallback)
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

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    Corners::all(active_indicator_radius(theme))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.active-indicator.color",
        "md.sys.color.secondary-container",
    )
}

pub(crate) fn scrim_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.scrim.color",
        "md.sys.color.scrim",
    )
}

pub(crate) fn scrim_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme)
        .number_optional(Some("md.comp.navigation-drawer.scrim.opacity"), 0.4)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    MaterialTokenResolver::new(theme).state_layer_opacity(
        "md.comp.navigation-drawer.pressed.state-layer.opacity",
        MaterialStateLayerInteraction::Pressed,
    )
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationDrawerItemInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    let Some((key, interaction)) = state_layer_opacity_token(interaction) else {
        return 0.0;
    };
    MaterialTokenResolver::new(theme).state_layer_opacity(key, interaction)
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
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(label_color_key(active, interaction), fallback)
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
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(icon_color_key(active, interaction), fallback)
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
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(state_layer_color_key(active, interaction), fallback)
}

fn state_layer_opacity_token(
    interaction: NavigationDrawerItemInteraction,
) -> Option<(&'static str, MaterialStateLayerInteraction)> {
    match interaction {
        NavigationDrawerItemInteraction::Default => None,
        NavigationDrawerItemInteraction::Pressed => Some((
            "md.comp.navigation-drawer.pressed.state-layer.opacity",
            MaterialStateLayerInteraction::Pressed,
        )),
        NavigationDrawerItemInteraction::Focused => Some((
            "md.comp.navigation-drawer.focus.state-layer.opacity",
            MaterialStateLayerInteraction::Focused,
        )),
        NavigationDrawerItemInteraction::Hovered => Some((
            "md.comp.navigation-drawer.hover.state-layer.opacity",
            MaterialStateLayerInteraction::Hovered,
        )),
    }
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    let weight_key = if active {
        "md.comp.navigation-drawer.active.label-text.weight"
    } else {
        "md.comp.navigation-drawer.label-text.weight"
    };
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-large",
        weight_key,
        if active { 700.0 } else { 500.0 },
        TextIntent::Control,
    )
}

pub(crate) fn large_badge_label_text_style(theme: &Theme) -> TextStyle {
    typography::text_style_with_weight_fallback(
        theme,
        None,
        "md.sys.typescale.label-small",
        "md.comp.navigation-drawer.large-badge-label.weight",
        500.0,
        TextIntent::Control,
    )
}

pub(crate) fn large_badge_label_color(theme: &Theme) -> Color {
    MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.navigation-drawer.large-badge-label.color",
        "md.sys.color.on-surface-variant",
    )
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.navigation-drawer.icon.size")
        .unwrap_or(Px(24.0))
}
