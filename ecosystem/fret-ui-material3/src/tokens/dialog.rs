//! Typed token access for Material 3 dialogs.
//!
//! This module centralizes token key mapping and fallback chains so dialog outcomes remain stable
//! and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::theme::CubicBezier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DialogActionInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn scrim_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.sys.color.scrim")
        .unwrap_or_else(|| theme.color_required("md.sys.color.scrim"))
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.dialog.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-high"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container-high"))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.dialog.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-large"))
        .unwrap_or_else(|| Corners::all(Px(28.0)))
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.dialog.container.elevation")
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.dialog.container.shadow-color")
        .or_else(|| theme.color_by_key("md.sys.color.shadow"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.shadow"))
}

pub(crate) fn headline_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.dialog.headline.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"))
}

pub(crate) fn supporting_text_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.dialog.supporting-text.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn default_open_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.medium2")
        .unwrap_or(300)
}

pub(crate) fn default_close_duration_ms(theme: &Theme) -> u32 {
    theme
        .duration_ms_by_key("md.sys.motion.duration.medium2")
        .unwrap_or(300)
}

pub(crate) fn easing(theme: &Theme, easing_key: Option<&str>) -> CubicBezier {
    let key = easing_key.unwrap_or("md.sys.motion.easing.emphasized");
    theme.easing_by_key(key).unwrap_or(CubicBezier {
        x1: 0.0,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    })
}

pub(crate) fn panel_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges::all(Px(24.0))
}

pub(crate) fn action_height(theme: &Theme) -> Px {
    let _ = theme;
    Px(40.0)
}

pub(crate) fn action_padding(theme: &Theme) -> Edges {
    let _ = theme;
    Edges {
        left: Px(12.0),
        right: Px(12.0),
        top: Px(0.0),
        bottom: Px(0.0),
    }
}

pub(crate) fn action_corner_radii(theme: &Theme) -> Corners {
    let _ = theme;
    Corners::all(Px(9999.0))
}

fn action_label_color_key(interaction: DialogActionInteraction) -> &'static str {
    match interaction {
        DialogActionInteraction::Pressed => "md.comp.dialog.action.pressed.label-text.color",
        DialogActionInteraction::Hovered => "md.comp.dialog.action.hover.label-text.color",
        DialogActionInteraction::Focused => "md.comp.dialog.action.focus.label-text.color",
        DialogActionInteraction::Default => "md.comp.dialog.action.label-text.color",
    }
}

pub(crate) fn action_label_color(theme: &Theme, interaction: DialogActionInteraction) -> Color {
    theme
        .color_by_key(action_label_color_key(interaction))
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

fn action_state_layer_color_key(interaction: DialogActionInteraction) -> &'static str {
    match interaction {
        DialogActionInteraction::Pressed => "md.comp.dialog.action.pressed.state-layer.color",
        DialogActionInteraction::Hovered => "md.comp.dialog.action.hover.state-layer.color",
        DialogActionInteraction::Focused | DialogActionInteraction::Default => {
            "md.comp.dialog.action.focus.state-layer.color"
        }
    }
}

pub(crate) fn action_state_layer_color(
    theme: &Theme,
    interaction: DialogActionInteraction,
) -> Color {
    theme
        .color_by_key(action_state_layer_color_key(interaction))
        .or_else(|| theme.color_by_key("md.sys.color.primary"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.primary"))
}

fn action_state_layer_opacity_key(interaction: DialogActionInteraction) -> Option<&'static str> {
    match interaction {
        DialogActionInteraction::Pressed => {
            Some("md.comp.dialog.action.pressed.state-layer.opacity")
        }
        DialogActionInteraction::Hovered => Some("md.comp.dialog.action.hover.state-layer.opacity"),
        DialogActionInteraction::Focused => Some("md.comp.dialog.action.focus.state-layer.opacity"),
        DialogActionInteraction::Default => None,
    }
}

fn sys_state_layer_opacity_key(interaction: DialogActionInteraction) -> Option<&'static str> {
    match interaction {
        DialogActionInteraction::Pressed => Some("md.sys.state.pressed.state-layer-opacity"),
        DialogActionInteraction::Hovered => Some("md.sys.state.hover.state-layer-opacity"),
        DialogActionInteraction::Focused => Some("md.sys.state.focus.state-layer-opacity"),
        DialogActionInteraction::Default => None,
    }
}

pub(crate) fn action_state_layer_target_opacity(
    theme: &Theme,
    interaction: DialogActionInteraction,
) -> f32 {
    let Some(key) = action_state_layer_opacity_key(interaction) else {
        return 0.0;
    };
    let sys_key = sys_state_layer_opacity_key(interaction)
        .unwrap_or("md.sys.state.hover.state-layer-opacity");
    theme
        .number_by_key(key)
        .or_else(|| theme.number_by_key(sys_key))
        .unwrap_or(match interaction {
            DialogActionInteraction::Pressed => 0.1,
            DialogActionInteraction::Hovered => 0.08,
            DialogActionInteraction::Focused => 0.1,
            DialogActionInteraction::Default => 0.0,
        })
}

pub(crate) fn action_pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.dialog.action.pressed.state-layer.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.pressed.state-layer-opacity"))
        .unwrap_or(0.1)
}
