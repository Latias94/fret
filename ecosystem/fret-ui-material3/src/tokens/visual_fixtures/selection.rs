use super::*;
use crate::icon_button::{IconButtonSize, IconButtonVariant};
use crate::tokens::{
    checkbox, chip, filter_chip, icon_button as icon_button_tokens, input_chip, radio,
    segmented_button, slider, suggestion_chip, switch,
};

pub(super) fn run_checkbox_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_radio_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_switch_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_slider_case(case: &Case, theme: &Theme) {
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

pub(super) fn run_segmented_button_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = segmented_button_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "none" => assert!(
                actual_segmented_button_optional_color(
                    theme,
                    selected,
                    enabled,
                    interaction,
                    &assertion.role
                )
                .is_none(),
                "{}:{} expected no color outcome",
                case.id,
                assertion.role
            ),
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_segmented_button_color(
                    theme,
                    selected,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_segmented_button_color(
                    theme,
                    selected,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_segmented_button_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_segmented_button_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_segmented_button_text_style(theme, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_icon_button_case(case: &Case, theme: &Theme) {
    let variant = icon_button_variant(&case.input.variant, &case.id);
    let toggle = case.input.toggle;
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = icon_button_interaction(case.input.interaction.as_deref(), &case.id);
    let icon =
        icon_button_tokens::icon_color(theme, variant, toggle, selected, enabled, interaction);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_icon_button_color(
                    theme,
                    variant,
                    toggle,
                    selected,
                    enabled,
                    interaction,
                    icon,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_icon_button_color(
                    theme,
                    variant,
                    toggle,
                    selected,
                    enabled,
                    interaction,
                    icon,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_icon_button_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_icon_button_number(
                    theme,
                    variant,
                    toggle,
                    selected,
                    interaction,
                    &assertion.role,
                ),
                token_number(theme, require_token(assertion, "token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_chip_case(case: &Case, theme: &Theme) {
    let enabled = enabled_input(case);
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_chip_color(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_chip_color(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_chip_metric(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_chip_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_chip_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_chip_text_style(theme, &assertion.role),
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

pub(super) fn run_filter_chip_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_filter_chip_color(
                    theme,
                    &case.input.variant,
                    selected,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_filter_chip_color(
                    theme,
                    &case.input.variant,
                    selected,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_filter_chip_metric(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_filter_chip_number(theme, selected, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_filter_chip_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_filter_chip_text_style(theme, &assertion.role),
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

pub(super) fn run_input_chip_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_input_chip_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_input_chip_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_input_chip_metric(theme, enabled, interaction, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_input_chip_number(theme, selected, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_input_chip_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_input_chip_text_style(theme, &assertion.role),
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

pub(super) fn run_suggestion_chip_case(case: &Case, theme: &Theme) {
    let enabled = enabled_input(case);
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_color(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_color(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_metric(
                    theme,
                    &case.input.variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_suggestion_chip_text_style(theme, &assertion.role),
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

// Selection-family token outcome helpers live with the cases that exercise them.
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

fn actual_segmented_button_optional_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> Option<Color> {
    match role {
        "container_color" => segmented_button::container_background(theme, selected),
        _ => Some(actual_segmented_button_color(
            theme,
            selected,
            enabled,
            interaction,
            role,
        )),
    }
}

fn actual_segmented_button_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => segmented_button::container_background(theme, selected)
            .expect("segmented button container color requires selected state"),
        "outline_color" => segmented_button::outline_color(theme, enabled),
        "label_color" => segmented_button::label_color(theme, selected, enabled, interaction),
        "icon_color" => segmented_button::icon_color(theme, selected, enabled, interaction),
        "state_layer_color" => segmented_button::state_layer_color(
            theme,
            selected,
            interaction.expect("segmented button state layer color requires interaction"),
        ),
        other => panic!("unsupported segmented button color role {other}"),
    }
}

fn actual_segmented_button_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_height" => segmented_button::container_height(theme),
        "outline_width" => segmented_button::outline_width(theme),
        "shape" => segmented_button::shape_radius(theme),
        "icon_size" => segmented_button::icon_size(theme),
        other => panic!("unsupported segmented button metric role {other}"),
    }
}

fn actual_segmented_button_number(
    theme: &Theme,
    interaction: Option<segmented_button::SegmentedButtonInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => segmented_button::state_layer_opacity(
            theme,
            interaction.expect("segmented button state layer opacity requires interaction"),
        ),
        "pressed_state_layer_opacity" => segmented_button::pressed_state_layer_opacity(theme),
        other => panic!("unsupported segmented button number role {other}"),
    }
}

fn actual_segmented_button_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => segmented_button::label_text_style(theme),
        other => panic!("unsupported segmented button text style role {other}"),
    }
}

fn actual_icon_button_color(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    enabled: bool,
    interaction: Option<icon_button_tokens::IconButtonInteraction>,
    icon: Color,
    role: &str,
) -> Color {
    match role {
        "icon_color" => {
            icon_button_tokens::icon_color(theme, variant, toggle, selected, enabled, interaction)
        }
        "state_layer_color" => icon_button_tokens::state_layer_color(
            theme,
            variant,
            toggle,
            selected,
            enabled,
            interaction,
        ),
        "container_color" => icon_button_tokens::container_background(
            theme, variant, toggle, selected, enabled, icon,
        )
        .expect("icon button container color requires filled, tonal, or selected outlined variant"),
        "outline_color" => icon_button_tokens::outlined_outline_color(theme, enabled),
        other => panic!("unsupported icon button color role {other}"),
    }
}

fn actual_icon_button_metric(theme: &Theme, role: &str) -> Px {
    let size = icon_button_tokens::size_tokens(theme, IconButtonSize::Small);
    match role {
        "container_size" => size.container,
        "leading_space" => size.pad_left,
        "trailing_space" => size.pad_right,
        "icon_size" => size.icon_size,
        "outline_width" => size.outline_width,
        "container_shape" => Px(icon_button_tokens::container_shape_radius(theme)),
        "selected_container_shape" => {
            Px(icon_button_tokens::selected_container_shape_radius(theme))
        }
        "pressed_container_shape" => Px(icon_button_tokens::pressed_container_shape_radius(theme)),
        other => panic!("unsupported icon button metric role {other}"),
    }
}

fn actual_icon_button_number(
    theme: &Theme,
    variant: IconButtonVariant,
    toggle: bool,
    selected: bool,
    interaction: Option<icon_button_tokens::IconButtonInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => icon_button_tokens::state_layer_opacity(
            theme,
            variant,
            toggle,
            selected,
            interaction.expect("icon button state layer opacity requires interaction"),
        ),
        "pressed_state_layer_opacity" => {
            icon_button_tokens::pressed_state_layer_opacity(theme, variant, toggle, selected)
        }
        other => panic!("unsupported icon button number role {other}"),
    }
}

fn actual_chip_color(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => chip::label_color(theme, enabled, interaction),
        "leading_icon_color" => chip::leading_icon_color(theme, enabled, interaction),
        "state_layer_color" => chip::state_layer_color(theme, interaction),
        "container_color" if variant == "elevated" => {
            chip::elevated_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => chip::elevated_container_shadow_color(theme),
        "outline_color" if variant == "flat" => {
            chip::flat_outline(theme, enabled, interaction)
                .expect("assist chip flat outline should exist")
                .color
        }
        other => panic!("unsupported assist chip color role {other}"),
    }
}

fn actual_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => chip::container_height(theme),
        "leading_icon_size" => chip::leading_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            chip::flat_outline(theme, enabled, interaction)
                .expect("assist chip flat outline should exist")
                .width
        }
        other => panic!("unsupported assist chip metric role {other}"),
    }
}

fn actual_chip_number(theme: &Theme, interaction: Option<PressableInteraction>, role: &str) -> f32 {
    match role {
        "state_layer_opacity" => chip::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => chip::pressed_state_layer_opacity(theme),
        other => panic!("unsupported assist chip number role {other}"),
    }
}

fn actual_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => chip::container_shape(theme),
        other => panic!("unsupported assist chip corners role {other}"),
    }
}

fn actual_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => chip::label_text_style(theme),
        other => panic!("unsupported assist chip text style role {other}"),
    }
}

fn actual_filter_chip_color(
    theme: &Theme,
    variant: &str,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => filter_chip::label_color(theme, selected, enabled, interaction),
        "leading_icon_color" => {
            filter_chip::leading_icon_color(theme, selected, enabled, interaction)
        }
        "trailing_icon_color" => {
            filter_chip::trailing_icon_color(theme, selected, enabled, interaction)
        }
        "state_layer_color" => filter_chip::state_layer_color(theme, selected, interaction),
        "container_color" if variant == "elevated" => {
            filter_chip::elevated_container_background(theme, selected, enabled)
        }
        "container_color" if variant == "flat" && selected => {
            filter_chip::flat_selected_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => {
            filter_chip::elevated_container_shadow_color(theme)
        }
        "outline_color" if variant == "flat" && !selected => {
            filter_chip::flat_unselected_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported filter chip color role {other}"),
    }
}

fn actual_filter_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => filter_chip::container_height(theme),
        "leading_icon_size" => filter_chip::leading_icon_size(theme),
        "trailing_icon_size" => filter_chip::trailing_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            filter_chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            filter_chip::flat_unselected_outline(theme, enabled, interaction).width
        }
        other => panic!("unsupported filter chip metric role {other}"),
    }
}

fn actual_filter_chip_number(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => filter_chip::state_layer_opacity(theme, selected, interaction),
        "pressed_state_layer_opacity" => filter_chip::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported filter chip number role {other}"),
    }
}

fn actual_filter_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => filter_chip::container_shape(theme),
        other => panic!("unsupported filter chip corners role {other}"),
    }
}

fn actual_filter_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => filter_chip::label_text_style(theme),
        other => panic!("unsupported filter chip text style role {other}"),
    }
}

fn actual_input_chip_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => input_chip::label_color(theme, selected, enabled, interaction),
        "leading_icon_color" => {
            input_chip::leading_icon_color(theme, selected, enabled, interaction)
        }
        "trailing_icon_color" => {
            input_chip::trailing_icon_color(theme, selected, enabled, interaction)
        }
        "state_layer_color" => input_chip::state_layer_color(theme, selected, interaction),
        "container_color" if selected => input_chip::selected_container_background(theme, enabled),
        "outline_color" if !selected => {
            input_chip::unselected_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported input chip color role {other}"),
    }
}

fn actual_input_chip_metric(
    theme: &Theme,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => input_chip::container_height(theme),
        "leading_icon_size" => input_chip::leading_icon_size(theme),
        "trailing_icon_size" => input_chip::trailing_icon_size(theme),
        "outline_width" => input_chip::unselected_outline(theme, enabled, interaction).width,
        other => panic!("unsupported input chip metric role {other}"),
    }
}

fn actual_input_chip_number(
    theme: &Theme,
    selected: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => input_chip::state_layer_opacity(theme, selected, interaction),
        "pressed_state_layer_opacity" => input_chip::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported input chip number role {other}"),
    }
}

fn actual_input_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => input_chip::container_shape(theme),
        other => panic!("unsupported input chip corners role {other}"),
    }
}

fn actual_input_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => input_chip::label_text_style(theme),
        other => panic!("unsupported input chip text style role {other}"),
    }
}

fn actual_suggestion_chip_color(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "label_color" => suggestion_chip::label_color(theme, enabled, interaction),
        "leading_icon_color" => suggestion_chip::leading_icon_color(theme, enabled, interaction),
        "state_layer_color" => suggestion_chip::state_layer_color(theme, interaction),
        "container_color" if variant == "elevated" => {
            suggestion_chip::elevated_container_background(theme, enabled)
        }
        "shadow_color" if variant == "elevated" => {
            suggestion_chip::elevated_container_shadow_color(theme)
        }
        "outline_color" if variant == "flat" => {
            suggestion_chip::flat_outline(theme, enabled, interaction).color
        }
        other => panic!("unsupported suggestion chip color role {other}"),
    }
}

fn actual_suggestion_chip_metric(
    theme: &Theme,
    variant: &str,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_height" => suggestion_chip::container_height(theme),
        "leading_icon_size" => suggestion_chip::leading_icon_size(theme),
        "container_elevation" if variant == "elevated" => {
            suggestion_chip::elevated_container_elevation(theme, enabled, interaction)
        }
        "outline_width" if variant == "flat" => {
            suggestion_chip::flat_outline(theme, enabled, interaction).width
        }
        other => panic!("unsupported suggestion chip metric role {other}"),
    }
}

fn actual_suggestion_chip_number(
    theme: &Theme,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => suggestion_chip::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => suggestion_chip::pressed_state_layer_opacity(theme),
        other => panic!("unsupported suggestion chip number role {other}"),
    }
}

fn actual_suggestion_chip_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => suggestion_chip::container_shape(theme),
        other => panic!("unsupported suggestion chip corners role {other}"),
    }
}

fn actual_suggestion_chip_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "label_text_style" => suggestion_chip::label_text_style(theme),
        other => panic!("unsupported suggestion chip text style role {other}"),
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

fn segmented_button_interaction(
    value: Option<&str>,
    case_id: &str,
) -> Option<segmented_button::SegmentedButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(segmented_button::SegmentedButtonInteraction::Hovered),
        "focused" => Some(segmented_button::SegmentedButtonInteraction::Focused),
        "pressed" => Some(segmented_button::SegmentedButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported segmented button interaction {other}"),
    }
}

fn icon_button_interaction(
    value: Option<&str>,
    case_id: &str,
) -> Option<icon_button_tokens::IconButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(icon_button_tokens::IconButtonInteraction::Hovered),
        "focused" => Some(icon_button_tokens::IconButtonInteraction::Focused),
        "pressed" => Some(icon_button_tokens::IconButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported icon button interaction {other}"),
    }
}

fn icon_button_variant(value: &str, case_id: &str) -> IconButtonVariant {
    match value {
        "standard" => IconButtonVariant::Standard,
        "filled" => IconButtonVariant::Filled,
        "tonal" => IconButtonVariant::Tonal,
        "outlined" => IconButtonVariant::Outlined,
        other => panic!("{case_id}: unsupported icon button variant {other}"),
    }
}
