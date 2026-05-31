//! Typed token access for Material 3 navigation drawers.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::navigation_drawer::NavigationDrawerVariant;
use crate::tokens::navigation_common;

pub(crate) use navigation_common::NavigationItemInteraction as NavigationDrawerItemInteraction;

pub(crate) fn container_width(theme: &Theme) -> Px {
    navigation_common::drawer_container_width(theme)
}

#[allow(dead_code)]
pub(crate) fn active_indicator_width(theme: &Theme) -> Px {
    navigation_common::drawer_active_indicator_width(theme)
}

pub(crate) fn item_horizontal_padding(theme: &Theme) -> Px {
    navigation_common::drawer_item_horizontal_padding(theme)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    navigation_common::drawer_container_shape(theme)
}

pub(crate) fn container_background(theme: &Theme, variant: NavigationDrawerVariant) -> Color {
    navigation_common::drawer_container_background(theme, variant)
}

pub(crate) fn container_elevation(theme: &Theme, variant: NavigationDrawerVariant) -> Px {
    navigation_common::drawer_container_elevation(theme, variant)
}

pub(crate) fn active_indicator_height(theme: &Theme) -> Px {
    navigation_common::drawer_active_indicator_height(theme)
}

pub(crate) fn active_indicator_shape(theme: &Theme) -> Corners {
    navigation_common::drawer_active_indicator_shape(theme)
}

pub(crate) fn active_indicator_color(theme: &Theme) -> Color {
    navigation_common::drawer_active_indicator_color(theme)
}

pub(crate) fn scrim_color(theme: &Theme) -> Color {
    navigation_common::drawer_scrim_color(theme)
}

pub(crate) fn scrim_opacity(theme: &Theme) -> f32 {
    navigation_common::drawer_scrim_opacity(theme)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    navigation_common::drawer_pressed_state_layer_opacity(theme)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: NavigationDrawerItemInteraction,
) -> f32 {
    navigation_common::drawer_state_layer_target_opacity(theme, enabled, interaction)
}

pub(crate) fn state_layer_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_state_layer_color(theme, active, interaction)
}

pub(crate) fn label_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_label_color(theme, active, interaction)
}

pub(crate) fn icon_color(
    theme: &Theme,
    active: bool,
    interaction: NavigationDrawerItemInteraction,
) -> Color {
    navigation_common::drawer_icon_color(theme, active, interaction)
}

pub(crate) fn label_text_style(theme: &Theme, active: bool) -> TextStyle {
    navigation_common::drawer_label_text_style(theme, active)
}

pub(crate) fn large_badge_label_text_style(theme: &Theme) -> TextStyle {
    navigation_common::drawer_large_badge_label_text_style(theme)
}

pub(crate) fn large_badge_label_color(theme: &Theme) -> Color {
    navigation_common::drawer_large_badge_label_color(theme)
}

pub(crate) fn icon_size(theme: &Theme) -> Px {
    navigation_common::drawer_icon_size(theme)
}
