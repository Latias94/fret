//! Typed token access for Material 3 navigation bars.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::tokens::navigation_common;

pub(crate) use navigation_common::NavigationItemInteraction as NavigationBarItemInteraction;

pub(crate) fn container_height(theme: &Theme) -> Px {
    navigation_common::bar_container_height(theme)
}

pub(crate) fn container_background(theme: &Theme) -> Color {
    navigation_common::bar_container_background(theme)
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    navigation_common::bar_container_elevation(theme)
}

pub(crate) fn container_shadow_color(theme: &Theme) -> Color {
    navigation_common::bar_container_shadow_color(theme)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    navigation_common::bar_container_shape(theme)
}

pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    navigation_common::bar_active_indicator_width(theme)
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    navigation_common::bar_active_indicator_height(theme)
}

pub(crate) fn active_indicator_top_offset(theme: &Theme) -> Px {
    navigation_common::bar_active_indicator_top_offset(theme)
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    navigation_common::bar_active_indicator_color(theme)
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    navigation_common::bar_active_indicator_shape(theme)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    state_layer_opacity(theme, NavigationBarItemInteraction::Pressed)
}

pub(crate) fn state_layer_opacity(theme: &Theme, interaction: NavigationBarItemInteraction) -> f32 {
    navigation_common::bar_state_layer_opacity(theme, interaction)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationBarItemInteraction,
) -> f32 {
    navigation_common::bar_state_layer_target_opacity(theme, enabled, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    navigation_common::bar_state_layer_color(theme, active, interaction)
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    navigation_common::bar_icon_color(theme, active, interaction)
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationBarItemInteraction,
) -> Color {
    navigation_common::bar_label_color(theme, active, interaction)
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    navigation_common::bar_label_text_style(theme, active)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    navigation_common::bar_icon_size(theme)
}

pub(crate) fn item_gap(theme: &Theme) -> Px {
    navigation_common::bar_item_gap(theme)
}
