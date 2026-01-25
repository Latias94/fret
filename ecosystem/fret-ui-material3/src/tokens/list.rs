//! Typed token access for Material 3 lists.
//!
//! This module centralizes token key mapping and fallback chains so list visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::content::MaterialContentDefaults;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ListItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn one_line_container_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.one-line.container.height")
        .unwrap_or(Px(56.0))
}

pub(crate) fn item_container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.list.list-item.container.shape")
        .unwrap_or(Corners::all(Px(0.0)))
}

pub(crate) fn item_between_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.between-space")
        .unwrap_or(Px(12.0))
}

pub(crate) fn item_leading_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.leading-space")
        .unwrap_or(Px(16.0))
}

pub(crate) fn item_trailing_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.trailing-space")
        .unwrap_or(Px(16.0))
}

pub(crate) fn item_top_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.top-space")
        .unwrap_or(Px(10.0))
}

pub(crate) fn item_bottom_space(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.bottom-space")
        .unwrap_or(Px(10.0))
}

pub(crate) fn leading_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.leading-icon.size")
        .unwrap_or(Px(24.0))
}

pub(crate) fn trailing_icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.list.list-item.trailing-icon.size")
        .unwrap_or(Px(24.0))
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

pub(crate) fn selected_container_background(theme: &Theme, enabled: bool) -> Color {
    if enabled {
        return theme
            .color_by_key("md.comp.list.list-item.selected.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.secondary-container"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.secondary-container"));
    }

    let mut bg = theme
        .color_by_key("md.comp.list.list-item.selected.disabled.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
    let opacity = theme
        .number_by_key("md.comp.list.list-item.selected.disabled.container.opacity")
        .or_else(|| theme.number_by_key("md.sys.state.disabled.state-layer-opacity"))
        .unwrap_or(0.38);
    bg.a = (bg.a * opacity).clamp(0.0, 1.0);
    bg
}

pub(crate) fn item_outcomes(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: ListItemInteraction,
) -> (Color, Color, Color, f32) {
    let defaults = MaterialContentDefaults::on_surface(theme);

    let (label_key, icon_key, state_layer_key, opacity_key) = match (selected, interaction) {
        (true, ListItemInteraction::Pressed) => (
            "md.comp.list.list-item.selected.pressed.label-text.color",
            "md.comp.list.list-item.selected.pressed.leading-icon.color",
            "md.comp.list.list-item.selected.pressed.state-layer.color",
            "md.comp.list.list-item.selected.pressed.state-layer.opacity",
        ),
        (true, ListItemInteraction::Focused) => (
            "md.comp.list.list-item.selected.focus.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.focus.state-layer.color",
            "md.comp.list.list-item.selected.focus.state-layer.opacity",
        ),
        (true, ListItemInteraction::Hovered) => (
            "md.comp.list.list-item.selected.hover.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (true, ListItemInteraction::Default) => (
            "md.comp.list.list-item.selected.label-text.color",
            "md.comp.list.list-item.selected.leading-icon.color",
            "md.comp.list.list-item.selected.hover.state-layer.color",
            "md.comp.list.list-item.selected.hover.state-layer.opacity",
        ),
        (false, ListItemInteraction::Pressed) => (
            "md.comp.list.list-item.pressed.label-text.color",
            "md.comp.list.list-item.pressed.leading-icon.icon.color",
            "md.comp.list.list-item.pressed.state-layer.color",
            "md.comp.list.list-item.pressed.state-layer.opacity",
        ),
        (false, ListItemInteraction::Focused) => (
            "md.comp.list.list-item.focus.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.focus.state-layer.color",
            "md.comp.list.list-item.focus.state-layer.opacity",
        ),
        (false, ListItemInteraction::Hovered) => (
            "md.comp.list.list-item.hover.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
        (false, ListItemInteraction::Default) => (
            "md.comp.list.list-item.label-text.color",
            "md.comp.list.list-item.leading-icon.color",
            "md.comp.list.list-item.hover.state-layer.color",
            "md.comp.list.list-item.hover.state-layer.opacity",
        ),
    };

    let mut label = theme
        .color_by_key(label_key)
        .unwrap_or(defaults.content_color);
    let mut icon = theme
        .color_by_key(icon_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));
    let state_layer = theme
        .color_by_key(state_layer_key)
        .unwrap_or(defaults.content_color);
    let mut opacity = theme.number_by_key(opacity_key).unwrap_or(0.0);

    if interaction == ListItemInteraction::Default {
        opacity = 0.0;
    }

    if !enabled {
        let (
            disabled_label_key,
            disabled_label_opacity_key,
            disabled_icon_key,
            disabled_icon_opacity_key,
        ) = if selected {
            (
                "md.comp.list.list-item.selected.disabled.label-text.color",
                "md.comp.list.list-item.selected.disabled.label-text.opacity",
                "md.comp.list.list-item.selected.disabled.leading-icon.color",
                "md.comp.list.list-item.selected.disabled.leading-icon.opacity",
            )
        } else {
            (
                "md.comp.list.list-item.disabled.label-text.color",
                "md.comp.list.list-item.disabled.label-text.opacity",
                "md.comp.list.list-item.disabled.leading-icon.color",
                "md.comp.list.list-item.disabled.leading-icon.opacity",
            )
        };

        label = theme
            .color_by_key(disabled_label_key)
            .unwrap_or(defaults.content_color);
        icon = theme
            .color_by_key(disabled_icon_key)
            .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));

        let label_opacity = theme
            .number_by_key(disabled_label_opacity_key)
            .unwrap_or(defaults.disabled_opacity);
        let icon_opacity = theme
            .number_by_key(disabled_icon_opacity_key)
            .unwrap_or(defaults.disabled_opacity);
        label = alpha_mul(label, label_opacity);
        icon = alpha_mul(icon, icon_opacity);
        opacity = 0.0;
    }

    (label, icon, state_layer, opacity)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme, selected: bool) -> f32 {
    theme
        .number_by_key(if selected {
            "md.comp.list.list-item.selected.pressed.state-layer.opacity"
        } else {
            "md.comp.list.list-item.pressed.state-layer.opacity"
        })
        .unwrap_or(0.1)
}
