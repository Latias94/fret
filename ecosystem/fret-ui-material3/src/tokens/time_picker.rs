//! Typed token access for Material 3 time picker primitives.
//!
//! Reference: Material Web v30 `md.comp.time-picker.*` tokens.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::tokens::time_picker_common;

pub(crate) const COMPONENT_PREFIX: &str = "md.comp.time-picker";

pub(crate) fn container_color(theme: &Theme) -> Color {
    time_picker_common::container_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_elevation(theme: &Theme) -> Px {
    time_picker_common::container_elevation(theme, COMPONENT_PREFIX)
}

pub(crate) fn container_shape(theme: &Theme) -> Corners {
    time_picker_common::container_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn headline_style(theme: &Theme) -> TextStyle {
    time_picker_common::headline_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn headline_color(theme: &Theme) -> Color {
    time_picker_common::headline_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_size(theme: &Theme) -> Px {
    time_picker_common::clock_dial_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_background(theme: &Theme) -> Color {
    time_picker_common::clock_dial_background(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_shape(theme: &Theme) -> Corners {
    time_picker_common::clock_dial_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_label_text_style(theme: &Theme) -> TextStyle {
    time_picker_common::clock_dial_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_label_text_color(theme: &Theme, selected: bool) -> Color {
    time_picker_common::clock_dial_label_text_color(theme, COMPONENT_PREFIX, selected)
}

pub(crate) fn clock_dial_handle_size(theme: &Theme) -> Px {
    time_picker_common::clock_dial_handle_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_handle_color(theme: &Theme) -> Color {
    time_picker_common::clock_dial_handle_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_handle_shape(theme: &Theme) -> Corners {
    time_picker_common::clock_dial_handle_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_size(theme: &Theme) -> Px {
    time_picker_common::clock_dial_selector_center_size(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_color(theme: &Theme) -> Color {
    time_picker_common::clock_dial_selector_center_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_center_shape(theme: &Theme) -> Corners {
    time_picker_common::clock_dial_selector_center_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_track_width(theme: &Theme) -> Px {
    time_picker_common::clock_dial_selector_track_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn clock_dial_selector_track_color(theme: &Theme) -> Color {
    time_picker_common::clock_dial_selector_track_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_width(theme: &Theme) -> Px {
    time_picker_common::time_selector_container_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_height(theme: &Theme) -> Px {
    time_picker_common::time_selector_container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_shape(theme: &Theme) -> Corners {
    time_picker_common::time_selector_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_container_color(theme: &Theme, selected: bool) -> Color {
    time_picker_common::time_selector_container_color(theme, COMPONENT_PREFIX, selected)
}

pub(crate) fn time_selector_label_text_style(theme: &Theme) -> TextStyle {
    time_picker_common::time_selector_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_separator_style(theme: &Theme) -> TextStyle {
    time_picker_common::time_selector_separator_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn time_selector_separator_color(theme: &Theme) -> Color {
    time_picker_common::time_selector_separator_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn display_separator_width(theme: &Theme) -> Px {
    time_picker_common::display_separator_width(theme)
}

pub(crate) fn time_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    time_picker_common::time_selector_label_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn time_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    time_picker_common::time_selector_state_layer_color(
        theme,
        COMPONENT_PREFIX,
        selected,
        interaction,
    )
}

pub(crate) fn time_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    time_picker_common::time_selector_state_layer_opacity(theme, COMPONENT_PREFIX, interaction)
}

pub(crate) fn period_selector_container_width(theme: &Theme) -> Px {
    time_picker_common::period_selector_container_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_container_height(theme: &Theme) -> Px {
    time_picker_common::period_selector_container_height(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_shape(theme: &Theme) -> Corners {
    time_picker_common::period_selector_shape(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_width(theme: &Theme) -> Px {
    time_picker_common::period_selector_outline_width(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_outline_color(theme: &Theme) -> Color {
    time_picker_common::period_selector_outline_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_selected_container_color(theme: &Theme) -> Color {
    time_picker_common::period_selector_selected_container_color(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_text_style(theme: &Theme) -> TextStyle {
    time_picker_common::period_selector_label_text_style(theme, COMPONENT_PREFIX)
}

pub(crate) fn period_selector_label_color(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
) -> Color {
    time_picker_common::period_selector_label_color(theme, COMPONENT_PREFIX, selected, interaction)
}

pub(crate) fn period_selector_state_layer_color(
    theme: &Theme,
    selected: bool,
    interaction: PressableInteraction,
) -> Color {
    time_picker_common::period_selector_state_layer_color(
        theme,
        COMPONENT_PREFIX,
        selected,
        interaction,
    )
}

pub(crate) fn period_selector_state_layer_opacity(
    theme: &Theme,
    interaction: PressableInteraction,
) -> f32 {
    time_picker_common::period_selector_state_layer_opacity(theme, COMPONENT_PREFIX, interaction)
}
