use super::super::button::ButtonInteraction;
use super::*;
use crate::button::{ButtonSize, ButtonVariant};
use crate::card::CardVariant;
use crate::fab::{FabSize, FabVariant};
use crate::tokens::{badge, button, card, carousel_item, divider, fab, list, progress_indicator};

pub(super) fn run_button_case(case: &Case, theme: &Theme) {
    let variant = button_variant(&case.input.variant, &case.id);
    let enabled = case.input.enabled.unwrap_or(true);
    let interaction = button_interaction(case.input.interaction.as_deref(), &case.id);
    let label = button::label_color(theme, variant, enabled);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "none" => assert!(
                actual_button_optional_color(
                    theme,
                    variant,
                    enabled,
                    interaction,
                    label,
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
                actual_button_color(theme, variant, enabled, interaction, label, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_button_color(theme, variant, enabled, interaction, label, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_button_metric(theme, variant, enabled, interaction, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "metric_literal" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_button_metric(theme, variant, enabled, interaction, &assertion.role),
                Px(require_value(assertion)),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_button_number(theme, variant, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_button_text_style(theme, ButtonSize::Small, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_card_case(case: &Case, theme: &Theme) {
    let variant = card_variant(&case.input.variant, &case.id);
    let enabled = enabled_input(case);
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_card_color(theme, variant, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_card_color(theme, variant, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_card_metric(theme, variant, enabled, interaction, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_card_number(theme, variant, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_card_corners(theme, variant, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_badge_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_badge_color(theme, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_badge_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_badge_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_badge_text_style(theme, &assertion.role),
                control_text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                ),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_fab_case(case: &Case, theme: &Theme) {
    let (extended, variant, size) = fab_case_variant(&case.input.variant, &case.id);
    let enabled = enabled_input(case);
    let interaction = fab_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_fab_color(
                    theme,
                    extended,
                    variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_fab_color(
                    theme,
                    extended,
                    variant,
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
                actual_fab_metric(
                    theme,
                    extended,
                    size,
                    variant,
                    enabled,
                    interaction,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_fab_number(theme, extended, variant, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_fab_corners(theme, extended, size, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_fab_text_style(theme, size, variant, &assertion.role),
                control_text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                ),
            ),
            "text_style_alias" => assert_text_style_alias(theme, case, assertion),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_list_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let enabled = enabled_input(case);
    let interaction = list_interaction(case.input.interaction.as_deref(), &case.id);
    let expressive = case.is_expressive_scheme();

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_list_color(theme, selected, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_list_color(theme, selected, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_list_metric(theme, &case.input.variant, expressive, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_list_number(theme, selected, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_list_corners(
                    theme,
                    selected,
                    enabled,
                    interaction,
                    expressive,
                    &assertion.role,
                ),
                token_corners(theme, require_token(assertion, "token")),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_list_text_style(theme, selected, &assertion.role),
                control_text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                ),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_list_text_style(theme, selected, &assertion.role),
                control_text_style(theme, require_token(assertion, "source_token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_progress_indicator_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_progress_indicator_color(theme, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_progress_indicator_metric(theme, &case.input.variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_progress_indicator_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_divider_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_divider_color(theme, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_divider_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_carousel_item_case(case: &Case, theme: &Theme) {
    let with_outline = matches!(case.input.variant.as_str(), "with_outline" | "outlined");
    let enabled = enabled_input(case);
    let disabled = !enabled;
    let interaction = pressable_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_carousel_item_color(
                    theme,
                    with_outline,
                    disabled,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_carousel_item_color(
                    theme,
                    with_outline,
                    disabled,
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
                actual_carousel_item_metric(
                    theme,
                    with_outline,
                    disabled,
                    interaction,
                    &assertion.role,
                ),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_carousel_item_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_carousel_item_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

// Surface-family token outcome helpers live with the cases that exercise them.
fn actual_button_optional_color(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    label: Color,
    role: &str,
) -> Option<Color> {
    match role {
        "container_color" => button::container_background(theme, variant, enabled, label),
        _ => Some(actual_button_color(
            theme,
            variant,
            enabled,
            interaction,
            label,
            role,
        )),
    }
}

fn actual_button_color(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    label: Color,
    role: &str,
) -> Color {
    match role {
        "label_color" => label,
        "container_color" => button::container_background(theme, variant, enabled, label)
            .expect("expected button container color"),
        "icon_color" => button::icon_color(theme, variant, enabled, label, interaction),
        "state_layer_color" => button::state_layer_color(theme, variant, label, interaction),
        "shadow_color" => button::container_shadow_color(theme, variant),
        other => panic!("unsupported button color role {other}"),
    }
}

fn actual_button_metric(
    theme: &Theme,
    variant: ButtonVariant,
    enabled: bool,
    interaction: Option<ButtonInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => button::container_elevation(theme, variant, enabled, interaction),
        other => panic!("unsupported button metric role {other}"),
    }
}

fn actual_button_number(
    theme: &Theme,
    variant: ButtonVariant,
    interaction: Option<ButtonInteraction>,
    role: &str,
) -> f32 {
    match (role, interaction) {
        ("state_layer_opacity", Some(interaction)) => {
            button::state_layer_opacity(theme, variant, interaction)
        }
        ("state_layer_opacity", None) => panic!("state_layer_opacity requires interaction"),
        _ => panic!("unsupported button number role {role}"),
    }
}

fn actual_button_text_style(theme: &Theme, size: ButtonSize, role: &str) -> TextStyle {
    match role {
        "label_text_style" => button::label_text_style(theme, size),
        other => panic!("unsupported button text style role {other}"),
    }
}

fn actual_card_color(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => card::container_background(theme, variant, enabled),
        "container_shadow_color" => card::container_shadow_color(theme, variant),
        "outline_color" => {
            card::outline(theme, variant, enabled, interaction)
                .unwrap_or_else(|| panic!("expected card outline"))
                .color
        }
        "state_layer_color" => card::state_layer_color(theme, variant, interaction),
        other => panic!("unsupported card color role {other}"),
    }
}

fn actual_card_metric(
    theme: &Theme,
    variant: CardVariant,
    enabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => card::container_elevation(theme, variant, enabled, interaction),
        "outline_width" => {
            card::outline(theme, variant, enabled, interaction)
                .unwrap_or_else(|| panic!("expected card outline"))
                .width
        }
        other => panic!("unsupported card metric role {other}"),
    }
}

fn actual_card_number(
    theme: &Theme,
    variant: CardVariant,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => card::state_layer_opacity(theme, variant, interaction),
        "pressed_state_layer_opacity" => card::pressed_state_layer_opacity(theme, variant),
        other => panic!("unsupported card number role {other}"),
    }
}

fn actual_card_corners(theme: &Theme, variant: CardVariant, role: &str) -> Corners {
    match role {
        "container_shape" => card::container_shape(theme, variant),
        other => panic!("unsupported card corners role {other}"),
    }
}

fn actual_badge_color(theme: &Theme, role: &str) -> Color {
    match role {
        "dot_color" => badge::dot_color(theme),
        "large_color" => badge::large_color(theme),
        "large_label_color" => badge::large_label_color(theme),
        other => panic!("unsupported badge color role {other}"),
    }
}

fn actual_badge_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "dot_size" => badge::dot_size(theme),
        "large_size" => badge::large_size(theme),
        other => panic!("unsupported badge metric role {other}"),
    }
}

fn actual_badge_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "dot_shape" => badge::shape(theme),
        "large_shape" => badge::large_shape(theme),
        other => panic!("unsupported badge corners role {other}"),
    }
}

fn actual_badge_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "large_label_text_style" => badge::large_label_text_style(theme),
        other => panic!("unsupported badge text style role {other}"),
    }
}

fn actual_fab_color(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => fab::container_background(theme, extended, variant, enabled, false),
        "container_shadow_color" => fab::container_shadow_color(theme, extended, variant),
        "icon_color" => fab::icon_color(theme, extended, variant, enabled, interaction),
        "label_color" => fab::label_color(theme, variant, enabled, interaction),
        "state_layer_color" => fab::state_layer_color(
            theme,
            extended,
            variant,
            interaction.unwrap_or(fab::FabInteraction::Pressed),
        ),
        other => panic!("unsupported fab color role {other}"),
    }
}

fn actual_fab_metric(
    theme: &Theme,
    extended: bool,
    size: FabSize,
    variant: FabVariant,
    enabled: bool,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_size" => fab::container_size(theme, size),
        "icon_size" => fab::icon_size(theme, size),
        "extended_container_height" => fab::extended_container_height(theme, size),
        "extended_min_width" => fab::extended_min_width(theme, size),
        "extended_icon_size" => fab::extended_icon_size(theme, size),
        "extended_leading_space" => fab::extended_leading_space(theme, size, true),
        "extended_trailing_space" => fab::extended_trailing_space(theme, size),
        "extended_icon_label_space" => fab::extended_icon_label_space(theme, size),
        "container_elevation" => {
            fab::container_elevation(theme, extended, variant, enabled, false, interaction)
        }
        other => panic!("unsupported fab metric role {other}"),
    }
}

fn actual_fab_number(
    theme: &Theme,
    extended: bool,
    variant: FabVariant,
    interaction: Option<fab::FabInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => fab::state_layer_opacity(
            theme,
            extended,
            variant,
            interaction.unwrap_or(fab::FabInteraction::Pressed),
        ),
        "pressed_state_layer_opacity" => {
            fab::pressed_state_layer_opacity_for_variant(theme, extended, variant)
        }
        other => panic!("unsupported fab number role {other}"),
    }
}

fn actual_fab_corners(theme: &Theme, extended: bool, size: FabSize, role: &str) -> Corners {
    match (extended, role) {
        (false, "container_shape") => fab::container_shape(theme, size),
        (true, "container_shape") => fab::extended_container_shape(theme, size),
        _ => panic!("unsupported fab corners role {role}"),
    }
}

fn actual_fab_text_style(
    theme: &Theme,
    size: FabSize,
    variant: FabVariant,
    role: &str,
) -> TextStyle {
    match role {
        "label_text_style" => fab::extended_label_text_style(theme, size, variant),
        other => panic!("unsupported fab text style role {other}"),
    }
}

fn actual_list_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: list::ListItemInteraction,
    role: &str,
) -> Color {
    let (label, icon, state_layer, _) = list::item_outcomes(theme, selected, enabled, interaction);
    match role {
        "selected_container_color" => list::selected_container_background(theme, enabled),
        "label_color" => label,
        "leading_icon_color" | "icon_color" => icon,
        "state_layer_color" => state_layer,
        "supporting_text_color" => list::supporting_text_color(theme, enabled, selected),
        "overline_text_color" => list::overline_text_color(theme, enabled, selected),
        "trailing_supporting_text_color" => {
            list::trailing_supporting_text_color(theme, enabled, selected)
        }
        other => panic!("unsupported list color role {other}"),
    }
}

fn actual_list_metric(theme: &Theme, variant: &str, expressive: bool, role: &str) -> Px {
    match role {
        "container_height" => match variant {
            "one_line" => list::one_line_container_height(theme),
            "two_line" => list::two_line_container_height(theme),
            "three_line" => list::three_line_container_height(theme),
            other => panic!("unsupported list variant {other}"),
        },
        "item_between_space" => list::item_between_space(theme),
        "item_leading_space" => list::item_leading_space(theme),
        "item_trailing_space" => list::item_trailing_space(theme),
        "item_top_space" => list::item_top_space(theme),
        "item_bottom_space" => list::item_bottom_space(theme),
        "leading_icon_size" => list::leading_icon_size_with_variant(theme, expressive),
        "trailing_icon_size" => list::trailing_icon_size_with_variant(theme, expressive),
        other => panic!("unsupported list metric role {other}"),
    }
}

fn actual_list_number(
    theme: &Theme,
    selected: bool,
    interaction: list::ListItemInteraction,
    role: &str,
) -> f32 {
    let (_, _, _, state_layer_opacity) = list::item_outcomes(theme, selected, true, interaction);
    match role {
        "state_layer_opacity" => state_layer_opacity,
        "pressed_state_layer_opacity" => list::pressed_state_layer_opacity(theme, selected),
        other => panic!("unsupported list number role {other}"),
    }
}

fn actual_list_corners(
    theme: &Theme,
    selected: bool,
    enabled: bool,
    interaction: list::ListItemInteraction,
    expressive: bool,
    role: &str,
) -> Corners {
    match role {
        "container_shape" => list::item_container_shape_for_interaction(
            theme,
            selected,
            enabled,
            interaction,
            expressive,
        ),
        other => panic!("unsupported list corners role {other}"),
    }
}

fn actual_list_text_style(theme: &Theme, selected: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => list::label_text_style(theme, selected),
        "supporting_text_style" => list::supporting_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        "overline_text_style" => list::overline_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        "trailing_supporting_text_style" => list::trailing_supporting_text_style(theme, selected)
            .map(|style| typography::with_intent(style, TextIntent::Control))
            .unwrap_or_default(),
        other => panic!("unsupported list text style role {other}"),
    }
}

fn actual_progress_indicator_color(theme: &Theme, role: &str) -> Color {
    match role {
        "track_color" => progress_indicator::track_color(theme),
        "active_color" => progress_indicator::active_color(theme),
        "four_color_1" => progress_indicator::four_color_palette(theme)[0],
        "four_color_2" => progress_indicator::four_color_palette(theme)[1],
        "four_color_3" => progress_indicator::four_color_palette(theme)[2],
        "four_color_4" => progress_indicator::four_color_palette(theme)[3],
        other => panic!("unsupported progress indicator color role {other}"),
    }
}

fn actual_progress_indicator_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        ("linear", "height") => progress_indicator::linear_height(theme),
        ("linear", "track_thickness") => progress_indicator::linear_track_thickness(theme),
        ("linear", "active_thickness") => progress_indicator::linear_active_thickness(theme),
        ("circular", "size") => progress_indicator::circular_size(theme),
        ("circular", "track_thickness") => progress_indicator::circular_track_thickness(theme),
        ("circular", "active_thickness") => progress_indicator::circular_active_thickness(theme),
        _ => panic!("unsupported progress indicator metric role {variant}:{role}"),
    }
}

fn actual_progress_indicator_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "track_shape" => progress_indicator::track_shape(theme),
        "active_shape" => progress_indicator::active_shape(theme),
        other => panic!("unsupported progress indicator corners role {other}"),
    }
}

fn actual_divider_color(theme: &Theme, role: &str) -> Color {
    match role {
        "color" | "divider_color" => divider::color(theme),
        other => panic!("unsupported divider color role {other}"),
    }
}

fn actual_divider_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "thickness" => divider::thickness(theme),
        other => panic!("unsupported divider metric role {other}"),
    }
}

fn actual_carousel_item_color(
    theme: &Theme,
    with_outline: bool,
    disabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Color {
    match role {
        "container_color" => carousel_item::container_background(theme, disabled),
        "container_shadow_color" => carousel_item::container_shadow_color(theme),
        "state_layer_color" => carousel_item::state_layer_color(theme, interaction),
        "outline_color" => {
            carousel_item::outline(theme, with_outline, disabled, interaction)
                .unwrap_or_else(|| panic!("expected carousel item outline"))
                .color
        }
        other => panic!("unsupported carousel item color role {other}"),
    }
}

fn actual_carousel_item_metric(
    theme: &Theme,
    with_outline: bool,
    disabled: bool,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> Px {
    match role {
        "container_elevation" => carousel_item::container_elevation(theme, disabled, interaction),
        "outline_width" => {
            carousel_item::outline(theme, with_outline, disabled, interaction)
                .unwrap_or_else(|| panic!("expected carousel item outline"))
                .width
        }
        other => panic!("unsupported carousel item metric role {other}"),
    }
}

fn actual_carousel_item_number(
    theme: &Theme,
    interaction: Option<PressableInteraction>,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => carousel_item::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => carousel_item::pressed_state_layer_opacity(theme),
        "disabled_opacity" => carousel_item::disabled_opacity(theme),
        other => panic!("unsupported carousel item number role {other}"),
    }
}

fn actual_carousel_item_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => carousel_item::container_shape(theme),
        other => panic!("unsupported carousel item corners role {other}"),
    }
}

fn button_variant(value: &str, case_id: &str) -> ButtonVariant {
    match value {
        "filled" => ButtonVariant::Filled,
        "tonal" => ButtonVariant::Tonal,
        "elevated" => ButtonVariant::Elevated,
        "outlined" => ButtonVariant::Outlined,
        "text" => ButtonVariant::Text,
        other => panic!("{case_id}: unsupported button variant {other}"),
    }
}

fn button_interaction(value: Option<&str>, case_id: &str) -> Option<ButtonInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(ButtonInteraction::Hovered),
        "focused" => Some(ButtonInteraction::Focused),
        "pressed" => Some(ButtonInteraction::Pressed),
        other => panic!("{case_id}: unsupported button interaction {other}"),
    }
}

fn card_variant(value: &str, case_id: &str) -> CardVariant {
    match value {
        "filled" => CardVariant::Filled,
        "elevated" => CardVariant::Elevated,
        "outlined" => CardVariant::Outlined,
        other => panic!("{case_id}: unsupported card variant {other}"),
    }
}

fn fab_case_variant(value: &str, case_id: &str) -> (bool, FabVariant, FabSize) {
    let mut extended = false;
    let mut variant = FabVariant::Surface;
    let mut size = FabSize::Regular;

    for part in value.split('_') {
        match part {
            "extended" => extended = true,
            "surface" => variant = FabVariant::Surface,
            "primary" => variant = FabVariant::Primary,
            "secondary" => variant = FabVariant::Secondary,
            "tertiary" => variant = FabVariant::Tertiary,
            "regular" => size = FabSize::Regular,
            "small" => size = FabSize::Small,
            "medium" => size = FabSize::Medium,
            "large" => size = FabSize::Large,
            "" => {}
            other => panic!("{case_id}: unsupported fab variant part {other}"),
        }
    }

    (extended, variant, size)
}

fn fab_interaction(value: Option<&str>, case_id: &str) -> Option<fab::FabInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(fab::FabInteraction::Hovered),
        "focused" => Some(fab::FabInteraction::Focused),
        "pressed" => Some(fab::FabInteraction::Pressed),
        other => panic!("{case_id}: unsupported fab interaction {other}"),
    }
}

fn list_interaction(value: Option<&str>, case_id: &str) -> list::ListItemInteraction {
    match value.unwrap_or("none") {
        "none" => list::ListItemInteraction::Default,
        "hovered" => list::ListItemInteraction::Hovered,
        "focused" => list::ListItemInteraction::Focused,
        "pressed" => list::ListItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported list interaction {other}"),
    }
}
