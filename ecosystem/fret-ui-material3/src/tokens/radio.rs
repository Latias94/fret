//! Typed token access for Material 3 radio buttons.
//!
//! This module centralizes token key mapping and fallback chains so radio visuals remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Px};
use fret_ui::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RadioInteraction {
    None,
    Hovered,
    Focused,
    Pressed,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RadioSizeTokens {
    pub(crate) icon: Px,
    pub(crate) state_layer: Px,
}

pub(crate) fn size_tokens(theme: &Theme) -> RadioSizeTokens {
    let icon = theme
        .metric_by_key("md.comp.radio-button.icon.size")
        .unwrap_or(Px(20.0));
    let state_layer = theme
        .metric_by_key("md.comp.radio-button.state-layer.size")
        .unwrap_or(Px(40.0));
    RadioSizeTokens { icon, state_layer }
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: RadioInteraction,
) -> f32 {
    if !enabled {
        return 0.0;
    }

    match interaction {
        RadioInteraction::None => 0.0,
        RadioInteraction::Pressed => theme
            .number_by_key(state_layer_opacity_key(checked, RadioInteraction::Pressed))
            .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
            .unwrap_or(0.1),
        RadioInteraction::Focused => theme
            .number_by_key(state_layer_opacity_key(checked, RadioInteraction::Focused))
            .or_else(|| theme.number_by_key("md.sys.state.focus.state-layer-opacity"))
            .unwrap_or(0.1),
        RadioInteraction::Hovered => theme
            .number_by_key(state_layer_opacity_key(checked, RadioInteraction::Hovered))
            .or_else(|| theme.number_by_key("md.sys.state.hover.state-layer-opacity"))
            .unwrap_or(0.08),
    }
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, checked: bool) -> f32 {
    theme
        .number_by_key(state_layer_opacity_key(checked, RadioInteraction::Pressed))
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    checked: bool,
    interaction: RadioInteraction,
) -> Color {
    theme
        .color_by_key(state_layer_color_key(checked, interaction))
        .unwrap_or_else(|| {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
        })
}

pub(crate) fn icon_color(
    theme: &Theme,
    checked: bool,
    enabled: bool,
    interaction: RadioInteraction,
) -> Color {
    if !enabled {
        let (color_key, opacity_key) = if checked {
            (
                "md.comp.radio-button.disabled.selected.icon.color",
                "md.comp.radio-button.disabled.selected.icon.opacity",
            )
        } else {
            (
                "md.comp.radio-button.disabled.unselected.icon.color",
                "md.comp.radio-button.disabled.unselected.icon.opacity",
            )
        };

        let base = theme
            .color_by_key(color_key)
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme.number_by_key(opacity_key).unwrap_or(0.38);
        return alpha_mul(base, opacity);
    }

    theme
        .color_by_key(icon_color_key(checked, interaction))
        .unwrap_or_else(|| {
            theme
                .color_by_key("md.sys.color.primary")
                .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
        })
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn state_layer_opacity_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::Pressed) => {
            "md.comp.radio-button.selected.pressed.state-layer.opacity"
        }
        (true, RadioInteraction::Focused) => {
            "md.comp.radio-button.selected.focus.state-layer.opacity"
        }
        (true, RadioInteraction::Hovered) => {
            "md.comp.radio-button.selected.hover.state-layer.opacity"
        }
        (false, RadioInteraction::Pressed) => {
            "md.comp.radio-button.unselected.pressed.state-layer.opacity"
        }
        (false, RadioInteraction::Focused) => {
            "md.comp.radio-button.unselected.focus.state-layer.opacity"
        }
        (false, RadioInteraction::Hovered) => {
            "md.comp.radio-button.unselected.hover.state-layer.opacity"
        }
        (_, RadioInteraction::None) => "md.comp.radio-button.unselected.hover.state-layer.opacity",
    }
}

fn state_layer_color_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::Pressed) => {
            "md.comp.radio-button.selected.pressed.state-layer.color"
        }
        (true, RadioInteraction::Focused) => {
            "md.comp.radio-button.selected.focus.state-layer.color"
        }
        (true, RadioInteraction::Hovered) => {
            "md.comp.radio-button.selected.hover.state-layer.color"
        }
        (true, RadioInteraction::None) => "md.comp.radio-button.selected.hover.state-layer.color",
        (false, RadioInteraction::Pressed) => {
            "md.comp.radio-button.unselected.pressed.state-layer.color"
        }
        (false, RadioInteraction::Focused) => {
            "md.comp.radio-button.unselected.focus.state-layer.color"
        }
        (false, RadioInteraction::Hovered) => {
            "md.comp.radio-button.unselected.hover.state-layer.color"
        }
        (false, RadioInteraction::None) => {
            "md.comp.radio-button.unselected.hover.state-layer.color"
        }
    }
}

fn icon_color_key(checked: bool, interaction: RadioInteraction) -> &'static str {
    match (checked, interaction) {
        (true, RadioInteraction::None) => "md.comp.radio-button.selected.icon.color",
        (true, RadioInteraction::Hovered) => "md.comp.radio-button.selected.hover.icon.color",
        (true, RadioInteraction::Focused) => "md.comp.radio-button.selected.focus.icon.color",
        (true, RadioInteraction::Pressed) => "md.comp.radio-button.selected.pressed.icon.color",
        (false, RadioInteraction::None) => "md.comp.radio-button.unselected.icon.color",
        (false, RadioInteraction::Hovered) => "md.comp.radio-button.unselected.hover.icon.color",
        (false, RadioInteraction::Focused) => "md.comp.radio-button.unselected.focus.icon.color",
        (false, RadioInteraction::Pressed) => "md.comp.radio-button.unselected.pressed.icon.color",
    }
}
