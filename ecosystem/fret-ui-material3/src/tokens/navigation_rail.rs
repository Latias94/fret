//! Typed token access for Material 3 navigation rails.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::tokens::navigation_common;

pub(crate) use navigation_common::NavigationItemInteraction as NavigationRailItemInteraction;

pub(crate) fn container_width(theme: &Theme) -> Px {
    navigation_common::rail_container_width(theme)
}

pub(crate) fn item_width(theme: &Theme) -> Px {
    navigation_common::rail_item_width(theme)
}

pub(crate) fn item_height(theme: &Theme) -> Px {
    navigation_common::rail_item_height(theme)
}

pub(crate) fn vertical_padding(theme: &Theme) -> Px {
    navigation_common::rail_vertical_padding(theme)
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    navigation_common::rail_container_background(theme)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    navigation_common::rail_container_shape(theme)
}

pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    navigation_common::rail_active_indicator_width(theme)
}

pub(crate) fn active_indicator_height(theme: &Theme, has_label: bool) -> Px {
    navigation_common::rail_active_indicator_height(theme, has_label)
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    navigation_common::rail_active_indicator_color(theme)
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    navigation_common::rail_active_indicator_shape(theme)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    state_layer_opacity(theme, NavigationRailItemInteraction::Pressed)
}

pub(crate) fn state_layer_opacity(
    theme: &Theme,
    interaction: NavigationRailItemInteraction,
) -> f32 {
    navigation_common::rail_state_layer_opacity(theme, interaction)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationRailItemInteraction,
) -> f32 {
    navigation_common::rail_state_layer_target_opacity(theme, enabled, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationRailItemInteraction,
) -> Color {
    navigation_common::rail_state_layer_color(theme, active, interaction)
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationRailItemInteraction,
) -> Color {
    navigation_common::rail_icon_color(theme, active, interaction)
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationRailItemInteraction,
) -> Color {
    navigation_common::rail_label_color(theme, active, interaction)
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    navigation_common::rail_label_text_style(theme, active)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    navigation_common::rail_icon_size(theme)
}
