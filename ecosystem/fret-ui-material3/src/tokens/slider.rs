//! Typed token access for Material 3 sliders.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::tokens::slider_common;

pub(crate) use slider_common::SliderInteraction;

pub(crate) fn state_layer_size(theme: &Theme) -> Px {
    slider_common::state_layer_size(theme)
}

pub(crate) fn state_layer_target_opacity(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> f32 {
    slider_common::state_layer_target_opacity(theme, enabled, interaction)
}

pub(crate) fn pressed_state_layer_opacity(theme: &Theme) -> f32 {
    slider_common::pressed_state_layer_opacity(theme)
}

pub(crate) fn state_layer_color(theme: &Theme, interaction: SliderInteraction) -> Color {
    slider_common::state_layer_color(theme, interaction)
}

pub(crate) fn value_indicator_bottom_space(theme: &Theme) -> Px {
    slider_common::value_indicator_bottom_space(theme)
}

pub(crate) fn value_indicator_container_color(theme: &Theme) -> Color {
    slider_common::value_indicator_container_color(theme)
}

pub(crate) fn value_indicator_label_color(theme: &Theme) -> Color {
    slider_common::value_indicator_label_color(theme)
}

pub(crate) fn value_indicator_label_style(theme: &Theme) -> TextStyle {
    slider_common::value_indicator_label_style(theme)
}

pub(crate) fn tick_mark_size(theme: &Theme) -> Px {
    slider_common::tick_mark_size(theme)
}

pub(crate) fn tick_mark_shape(theme: &Theme) -> Corners {
    slider_common::tick_mark_shape(theme)
}

pub(crate) fn tick_mark_color(theme: &Theme, enabled: bool, active: bool) -> Color {
    slider_common::tick_mark_color(theme, enabled, active)
}

pub(crate) fn tick_mark_opacity(theme: &Theme, enabled: bool, active: bool) -> f32 {
    slider_common::tick_mark_opacity(theme, enabled, active)
}

pub(crate) fn stop_indicator_size(theme: &Theme) -> Px {
    slider_common::stop_indicator_size(theme)
}

pub(crate) fn stop_indicator_shape(theme: &Theme) -> Corners {
    slider_common::stop_indicator_shape(theme)
}

pub(crate) fn stop_indicator_trailing_space(theme: &Theme) -> Px {
    slider_common::stop_indicator_trailing_space(theme)
}

pub(crate) fn stop_indicator_color(theme: &Theme, enabled: bool, selected: bool) -> Color {
    slider_common::stop_indicator_color(theme, enabled, selected)
}

pub(crate) fn active_track_height(theme: &Theme) -> Px {
    slider_common::active_track_height(theme)
}

pub(crate) fn inactive_track_height(theme: &Theme) -> Px {
    slider_common::inactive_track_height(theme)
}

pub(crate) fn active_track_color(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> Color {
    slider_common::active_track_color(theme, enabled, interaction)
}

pub(crate) fn inactive_track_color(
    theme: &Theme,
    enabled: bool,
    interaction: SliderInteraction,
) -> Color {
    slider_common::inactive_track_color(theme, enabled, interaction)
}

pub(crate) fn handle_color(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Color {
    slider_common::handle_color(theme, enabled, interaction)
}

pub(crate) fn track_shape(theme: &Theme) -> Corners {
    slider_common::track_shape(theme)
}

pub(crate) fn handle_height(theme: &Theme) -> Px {
    slider_common::handle_height(theme)
}

pub(crate) fn handle_width(theme: &Theme, enabled: bool, interaction: SliderInteraction) -> Px {
    slider_common::handle_width(theme, enabled, interaction)
}

pub(crate) fn handle_shape(theme: &Theme) -> Corners {
    slider_common::handle_shape(theme)
}
