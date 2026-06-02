use super::*;

pub(super) fn run_menu_case(case: &Case, theme: &Theme) {
    let enabled = enabled_input(case);
    let interaction = menu_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_menu_color(theme, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_menu_color(theme, enabled, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_menu_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_menu_number(theme, enabled, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "number_literal" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_menu_number(theme, enabled, interaction, &assertion.role),
                require_value(assertion),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_menu_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_menu_text_style(theme, &assertion.role),
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

pub(super) fn run_dropdown_menu_case(case: &Case, theme: &Theme) {
    let enabled = enabled_input(case);
    let interaction = menu_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_menu_color(theme, enabled, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_dropdown_menu_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_menu_number(theme, enabled, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_menu_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_menu_text_style(theme, &assertion.role),
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

pub(super) fn run_dialog_case(case: &Case, theme: &Theme) {
    let interaction = dialog_action_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_dialog_color(theme, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_dialog_color(theme, interaction, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_dialog_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_dialog_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_dialog_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_dialog_text_style(theme, &assertion.role),
                text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                    text_intent_for_role(&assertion.role),
                ),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_bottom_sheet_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_bottom_sheet_color(theme, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_bottom_sheet_color(theme, &assertion.role),
                color_with_alpha(
                    theme,
                    require_token(assertion, "color_token"),
                    require_token(assertion, "opacity_token"),
                ),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_bottom_sheet_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_bottom_sheet_number(theme, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_bottom_sheet_corners(theme, &assertion.role),
                token_corners(theme, require_token(assertion, "token")),
            ),
            other => panic!(
                "{}:{} unsupported assertion kind {other}",
                case.id, assertion.role
            ),
        }
    }
}

pub(super) fn run_tooltip_case(case: &Case, theme: &Theme) {
    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_tooltip_color(theme, &case.input.variant, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_tooltip_metric(theme, &case.input.variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "metric_literal" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_tooltip_metric(theme, &case.input.variant, &assertion.role),
                Px(require_value(assertion)),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_tooltip_corners(theme, &case.input.variant, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_tooltip_text_style(theme, &case.input.variant, &assertion.role),
                content_text_style_with_weight(
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

pub(super) fn run_snackbar_case(case: &Case, theme: &Theme) {
    let interaction = snackbar_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_snackbar_color(theme, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_snackbar_metric(theme, &case.input.variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_snackbar_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_snackbar_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_snackbar_text_style(theme, &assertion.role),
                text_style_with_weight(
                    theme,
                    require_token(assertion, "source_token"),
                    require_token(assertion, "token"),
                    text_intent_for_role(&assertion.role),
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
