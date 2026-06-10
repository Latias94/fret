use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::tokens::{checkbox, radio, slider, switch};

use super::super::super::visual_fixture_model::Case;
use super::super::assertions::*;
use super::super::input::enabled_input;
use super::super::token_lookup::*;
use super::super::typography_helpers::{control_text_style, control_text_style_with_weight};
pub(in super::super) fn run_checkbox_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = checkbox_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_checkbox_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_checkbox_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_checkbox_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_checkbox_number(theme, selected, enabled, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(in super::super) fn run_radio_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = radio_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_radio_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_radio_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_radio_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_radio_number(theme, selected, enabled, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(in super::super) fn run_switch_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = switch_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_switch_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_switch_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_switch_metric(theme, selected, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_switch_number(theme, selected, enabled, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_switch_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(in super::super) fn run_slider_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = slider_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_slider_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_slider_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_slider_metric(theme, enabled, interaction, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_slider_number(theme, enabled, selected, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_slider_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_slider_text_style(theme, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_slider_text_style(theme, &assertion.role),
                control_text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                ),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

fn actual_checkbox_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: checkbox::CheckboxInteraction,
    role: &str,
) -> Color {
    let chrome = checkbox::chrome(theme, selected, enabled, interaction);
    match role {
        "container_color" => chrome
            .container_bg
            .expect("checkbox container color requires selected state"),
        "outline_color" => chrome
            .outline_color
            .expect("checkbox outline color requires unselected state"),
        "icon_color" => chrome.icon_color,
        "state_layer_color" => checkbox::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported checkbox color role {other}"),
    }
}

fn actual_checkbox_metric(theme: &Theme, role: &str) -> Px {
    let size = checkbox::size_tokens(theme);
    match role {
        "container_size" => size.container,
        "icon_size" => size.icon,
        "state_layer_size" => size.state_layer,
        "container_shape" => size.container_corner,
        other => panic!("unsupported checkbox metric role {other}"),
    }
}

fn actual_checkbox_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: checkbox::CheckboxInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            checkbox::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => checkbox::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported checkbox number role {other}"),
    }
}

fn actual_radio_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: radio::RadioInteraction,
    role: &str,
) -> Color {
    match role {
        "icon_color" => radio::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => radio::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported radio color role {other}"),
    }
}

fn actual_radio_metric(theme: &Theme, role: &str) -> Px {
    let size = radio::size_tokens(theme);
    match role {
        "icon_size" => size.icon,
        "state_layer_size" => size.state_layer,
        other => panic!("unsupported radio metric role {other}"),
    }
}

fn actual_radio_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: radio::RadioInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            radio::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => radio::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported radio number role {other}"),
    }
}

fn actual_switch_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: switch::SwitchInteraction,
    role: &str,
) -> Color {
    let chrome = switch::chrome(theme, selected, enabled, interaction);
    match role {
        "track_color" => chrome.track_color,
        "outline_color" => chrome
            .outline_color
            .expect("switch outline color requires unselected state"),
        "handle_color" => chrome.handle_color,
        "icon_color" => switch::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => switch::state_layer_color(theme, selected, interaction),
        other => panic!("unsupported switch color role {other}"),
    }
}

fn actual_switch_metric(theme: &Theme, selected: bool, role: &str) -> Px {
    match role {
        "icon_size" => switch::icon_size(theme, selected),
        other => panic!("unsupported switch metric role {other}"),
    }
}

fn actual_switch_number(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: switch::SwitchInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            switch::state_layer_target_opacity(theme, selected, enabled, interaction)
        }
        "pressed_state_layer_opacity" => switch::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported switch number role {other}"),
    }
}

fn actual_switch_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "track_shape" => switch::track_shape(theme),
        "handle_shape" => switch::handle_shape(theme),
        "state_layer_shape" => switch::state_layer_shape(theme),
        other => panic!("unsupported switch corners role {other}"),
    }
}

fn actual_slider_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> Color {
    match role {
        "state_layer_color" => slider::state_layer_color(theme, interaction),
        "value_indicator_container_color" => slider::value_indicator_container_color(theme),
        "value_indicator_label_color" => slider::value_indicator_label_color(theme),
        "tick_mark_color" => slider::tick_mark_color(theme, enabled, selected),
        "stop_indicator_color" => slider::stop_indicator_color(theme, enabled, selected),
        "active_track_color" => slider::active_track_color(theme, enabled, interaction),
        "inactive_track_color" => slider::inactive_track_color(theme, enabled, interaction),
        "handle_color" => slider::handle_color(theme, enabled, interaction),
        other => panic!("unsupported slider color role {other}"),
    }
}

fn actual_slider_metric(
    theme: &Theme,
    enabled: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> Px {
    match role {
        "state_layer_size" => slider::state_layer_size(theme),
        "value_indicator_bottom_space" => slider::value_indicator_bottom_space(theme),
        "tick_mark_size" => slider::tick_mark_size(theme),
        "stop_indicator_size" => slider::stop_indicator_size(theme),
        "stop_indicator_trailing_space" => slider::stop_indicator_trailing_space(theme),
        "active_track_height" => slider::active_track_height(theme),
        "inactive_track_height" => slider::inactive_track_height(theme),
        "handle_height" => slider::handle_height(theme),
        "handle_width" => slider::handle_width(theme, enabled, interaction),
        other => panic!("unsupported slider metric role {other}"),
    }
}

fn actual_slider_number(
    theme: &Theme,
    enabled: bool,
    selected: bool,
    interaction: slider::SliderInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => slider::state_layer_target_opacity(theme, enabled, interaction),
        "pressed_state_layer_opacity" => slider::pressed_state_layer_opacity(theme),
        "tick_mark_opacity" => slider::tick_mark_opacity(theme, enabled, selected),
        other => panic!("unsupported slider number role {other}"),
    }
}

fn actual_slider_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "tick_mark_shape" => slider::tick_mark_shape(theme),
        "stop_indicator_shape" => slider::stop_indicator_shape(theme),
        "track_shape" => slider::track_shape(theme),
        "handle_shape" => slider::handle_shape(theme),
        other => panic!("unsupported slider corners role {other}"),
    }
}

fn actual_slider_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "value_indicator_label_style" => slider::value_indicator_label_style(theme),
        other => panic!("unsupported slider text style role {other}"),
    }
}

fn checkbox_interaction(value: Option<&str>, case_id: &str) -> checkbox::CheckboxInteraction {
    match value.unwrap_or("none") {
        "none" => checkbox::CheckboxInteraction::None,
        "hovered" => checkbox::CheckboxInteraction::Hovered,
        "focused" => checkbox::CheckboxInteraction::Focused,
        "pressed" => checkbox::CheckboxInteraction::Pressed,
        other => panic!("{case_id}: unsupported checkbox interaction {other}"),
    }
}

fn radio_interaction(value: Option<&str>, case_id: &str) -> radio::RadioInteraction {
    match value.unwrap_or("none") {
        "none" => radio::RadioInteraction::None,
        "hovered" => radio::RadioInteraction::Hovered,
        "focused" => radio::RadioInteraction::Focused,
        "pressed" => radio::RadioInteraction::Pressed,
        other => panic!("{case_id}: unsupported radio interaction {other}"),
    }
}

fn switch_interaction(value: Option<&str>, case_id: &str) -> switch::SwitchInteraction {
    match value.unwrap_or("none") {
        "none" => switch::SwitchInteraction::None,
        "hovered" => switch::SwitchInteraction::Hovered,
        "focused" => switch::SwitchInteraction::Focused,
        "pressed" => switch::SwitchInteraction::Pressed,
        other => panic!("{case_id}: unsupported switch interaction {other}"),
    }
}

fn slider_interaction(value: Option<&str>, case_id: &str) -> slider::SliderInteraction {
    match value.unwrap_or("none") {
        "none" => slider::SliderInteraction::None,
        "hovered" => slider::SliderInteraction::Hovered,
        "focused" => slider::SliderInteraction::Focused,
        "pressed" => slider::SliderInteraction::Pressed,
        other => panic!("{case_id}: unsupported slider interaction {other}"),
    }
}
