use fret_core::{Color, Px, TextStyle};
use fret_ui::Theme;

use crate::icon_button::{IconButtonSize, IconButtonVariant};
use crate::tokens::{icon_button as icon_button_tokens, segmented_button};

use super::super::super::visual_fixture_model::Case;
use super::super::assertions::*;
use super::super::input::enabled_input;
use super::super::token_lookup::*;
use super::super::typography_helpers::control_text_style;
pub(in super::super) fn run_segmented_button_case(case: &Case, theme: &Theme) {
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

pub(in super::super) fn run_icon_button_case(case: &Case, theme: &Theme) {
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
