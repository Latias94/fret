use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::foundation::interaction::PressableInteraction;
use crate::tokens::{chip, filter_chip, input_chip, suggestion_chip};

use super::super::super::visual_fixture_model::Case;
use super::super::assertions::*;
use super::super::input::{enabled_input, pressable_interaction};
use super::super::token_lookup::*;
use super::super::typography_helpers::control_text_style_with_weight;
pub(in super::super) fn run_chip_case(case: &Case, theme: &Theme) {
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

pub(in super::super) fn run_filter_chip_case(case: &Case, theme: &Theme) {
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

pub(in super::super) fn run_input_chip_case(case: &Case, theme: &Theme) {
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

pub(in super::super) fn run_suggestion_chip_case(case: &Case, theme: &Theme) {
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
