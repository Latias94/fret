//! Typed token access for Material 3 menus.
//!
//! This module centralizes token key mapping and fallback chains so menu visuals remain stable and
//! drift-resistant during refactors.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::content::MaterialContentDefaults;

pub(crate) const ITEM_MIN_WIDTH_FALLBACK: Px = Px(112.0);
pub(crate) const ITEM_MAX_WIDTH_FALLBACK: Px = Px(280.0);
pub(crate) const CONTAINER_VERTICAL_PADDING_FALLBACK: Px = Px(8.0);
pub(crate) const ITEM_HORIZONTAL_PADDING_FALLBACK: Px = Px(12.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MenuItemInteraction {
    Default,
    Hovered,
    Focused,
    Pressed,
}

pub(crate) fn list_item_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.height")
        .unwrap_or(Px(48.0))
}

pub(crate) fn item_min_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.min-width")
        .unwrap_or(ITEM_MIN_WIDTH_FALLBACK)
}

pub(crate) fn item_max_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.container.max-width")
        .unwrap_or(ITEM_MAX_WIDTH_FALLBACK)
}

pub(crate) fn container_vertical_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.container.vertical-padding")
        .unwrap_or(CONTAINER_VERTICAL_PADDING_FALLBACK)
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.list-item.content.horizontal-padding")
        .unwrap_or(ITEM_HORIZONTAL_PADDING_FALLBACK)
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.menu.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.surface-container"))
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.container.elevation")
        .unwrap_or(Px(0.0))
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.menu.container.shadow-color")
        .or_else(|| theme.color_by_key("md.sys.color.shadow"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.shadow"))
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.menu.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

pub(crate) fn divider_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("md.comp.menu.divider.height")
        .unwrap_or(Px(1.0))
}

pub(crate) fn divider_color(theme: &Theme) -> Color {
    theme
        .color_by_key("md.comp.menu.divider.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-variant"))
        .unwrap_or_else(|| theme.color_token("md.sys.color.surface-variant"))
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    theme
        .number_by_key("md.comp.menu.list-item.pressed.state-layer.opacity")
        .unwrap_or(0.1)
}

pub(crate) fn item_outcomes(
    theme: &Theme,
    enabled: bool,
    interaction: MenuItemInteraction,
) -> (Color, Color, f32) {
    let (label_key, state_layer_key, opacity_key) = match interaction {
        MenuItemInteraction::Pressed => (
            "md.comp.menu.list-item.pressed.label-text.color",
            "md.comp.menu.list-item.pressed.state-layer.color",
            "md.comp.menu.list-item.pressed.state-layer.opacity",
        ),
        MenuItemInteraction::Focused => (
            "md.comp.menu.list-item.focus.label-text.color",
            "md.comp.menu.list-item.focus.state-layer.color",
            "md.comp.menu.list-item.focus.state-layer.opacity",
        ),
        MenuItemInteraction::Hovered => (
            "md.comp.menu.list-item.hover.label-text.color",
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        ),
        MenuItemInteraction::Default => (
            "md.comp.menu.list-item.label-text.color",
            // Keep the default state-layer token aligned to hover for this MVP.
            "md.comp.menu.list-item.hover.state-layer.color",
            "md.comp.menu.list-item.hover.state-layer.opacity",
        ),
    };

    let defaults = MaterialContentDefaults::on_surface(theme);
    let mut label = theme
        .color_by_key(label_key)
        .unwrap_or(defaults.content_color);
    let state_layer = theme
        .color_by_key(state_layer_key)
        .unwrap_or(defaults.content_color);
    let mut opacity = theme.number_by_key(opacity_key).unwrap_or(0.0);

    if !enabled {
        let label_opacity = theme
            .number_by_key("md.comp.menu.list-item.disabled.label-text.opacity")
            .unwrap_or(defaults.disabled_opacity);
        label.a = (label.a * label_opacity).clamp(0.0, 1.0);
        opacity = 0.0;
    }

    (label, state_layer, opacity)
}
