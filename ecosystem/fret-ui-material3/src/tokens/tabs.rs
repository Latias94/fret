//! Typed token access for Material 3 primary navigation tabs.
//!
//! This module centralizes token key mapping and fallback chains so tab visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TabInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.primary-navigation-tab.container.height")
        .unwrap_or(Px(48.0))
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.primary-navigation-tab.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container"))
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.primary-navigation-tab.active-indicator.height")
        .unwrap_or(Px(3.0))
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.primary-navigation-tab.active-indicator.color")
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.primary-navigation-tab.active-indicator.shape")
        .unwrap_or(Corners {
            top_left: Px(3.0),
            top_right: Px(3.0),
            bottom_right: Px(0.0),
            bottom_left: Px(0.0),
        })
}

pub(crate) fn label_color(theme: &Theme, active: bool, interaction: TabInteraction) -> Color {
    theme
        .color_by_key(label_color_key(active, interaction))
        .or_else(|| {
            if active {
                theme.color_by_key("md.sys.color.primary")
            } else {
                theme.color_by_key("md.sys.color.on-surface-variant")
            }
        })
        .unwrap_or_else(|| {
            if active {
                theme.color_required("md.sys.color.primary")
            } else {
                theme.color_required("md.sys.color.on-surface-variant")
            }
        })
}

pub(crate) fn state_layer_color(theme: &Theme, active: bool, interaction: TabInteraction) -> Color {
    theme
        .color_by_key(state_layer_color_key(active, interaction))
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"))
}

pub(crate) fn state_layer_opacity(theme: &Theme, active: bool, interaction: TabInteraction) -> f32 {
    match interaction {
        TabInteraction::Default => 0.0,
        TabInteraction::Pressed => theme
            .number_by_key(state_layer_opacity_key(active, TabInteraction::Pressed))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        TabInteraction::Focused => theme
            .number_by_key(state_layer_opacity_key(active, TabInteraction::Focused))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        TabInteraction::Hovered => theme
            .number_by_key(state_layer_opacity_key(active, TabInteraction::Hovered))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, active: bool) -> f32 {
    state_layer_opacity(theme, active, TabInteraction::Pressed)
}

fn label_color_key(active: bool, interaction: TabInteraction) -> &'static str {
    match (active, interaction) {
        (true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-label-text.active.focus.label-text.color"
        }
        (true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-label-text.active.hover.label-text.color"
        }
        (true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-label-text.active.pressed.label-text.color"
        }
        (true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-label-text.active.label-text.color"
        }
        (false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.focus.label-text.color"
        }
        (false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.hover.label-text.color"
        }
        (false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.pressed.label-text.color"
        }
        (false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.with-label-text.inactive.label-text.color"
        }
    }
}

fn state_layer_color_key(active: bool, interaction: TabInteraction) -> &'static str {
    match (active, interaction) {
        (true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.active.focus.state-layer.color"
        }
        (true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.color"
        }
        (true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.active.pressed.state-layer.color"
        }
        (true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.color"
        }
        (false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.inactive.focus.state-layer.color"
        }
        (false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
        }
        (false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.inactive.pressed.state-layer.color"
        }
        (false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.color"
        }
    }
}

fn state_layer_opacity_key(active: bool, interaction: TabInteraction) -> &'static str {
    match (active, interaction) {
        (true, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.active.pressed.state-layer.opacity"
        }
        (true, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.active.focus.state-layer.opacity"
        }
        (true, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.opacity"
        }
        (true, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.active.hover.state-layer.opacity"
        }
        (false, TabInteraction::Pressed) => {
            "md.comp.primary-navigation-tab.inactive.pressed.state-layer.opacity"
        }
        (false, TabInteraction::Focused) => {
            "md.comp.primary-navigation-tab.inactive.focus.state-layer.opacity"
        }
        (false, TabInteraction::Hovered) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.opacity"
        }
        (false, TabInteraction::Default) => {
            "md.comp.primary-navigation-tab.inactive.hover.state-layer.opacity"
        }
    }
}
