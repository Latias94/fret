use super::*;

pub(super) fn run_tabs_case(case: &Case, theme: &Theme) {
    let selected = case.input.selected;
    let interaction = tab_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_tabs_color(theme, selected, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_tabs_metric(theme, &case.input.variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_tabs_number(theme, selected, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_tabs_corners(theme, &assertion.role),
                token_corners(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_tabs_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_tabs_text_style(theme, &assertion.role),
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

pub(super) fn run_navigation_bar_case(case: &Case, theme: &Theme) {
    let active = case.input.selected;
    let interaction = navigation_bar_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_navigation_bar_color(theme, active, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_navigation_bar_metric(theme, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_navigation_bar_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_navigation_bar_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_navigation_bar_text_style(theme, active, &assertion.role),
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

pub(super) fn run_navigation_rail_case(case: &Case, theme: &Theme) {
    let active = case.input.selected;
    let interaction = navigation_rail_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_navigation_rail_color(theme, active, interaction, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_navigation_rail_metric(theme, &case.input.variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_navigation_rail_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_navigation_rail_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_navigation_rail_text_style(theme, active, &assertion.role),
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

pub(super) fn run_navigation_drawer_case(case: &Case, theme: &Theme) {
    let variant = navigation_drawer_variant(&case.input.variant, &case.id);
    let active = case.input.selected;
    let interaction = navigation_drawer_interaction(case.input.interaction.as_deref(), &case.id);

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_color(
                    theme,
                    variant,
                    active,
                    interaction,
                    &assertion.role,
                ),
                token_color(theme, require_token(assertion, "token")),
            ),
            "color_alpha" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_color(
                    theme,
                    variant,
                    active,
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
                actual_navigation_drawer_metric(theme, variant, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "number" => assert_number_close(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_number(theme, interaction, &assertion.role),
                token_number(theme, require_token(assertion, "token")),
            ),
            "corners" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_corners(theme, &assertion.role),
                token_corners(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_corners(theme, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_weight_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_navigation_drawer_text_style(theme, active, &assertion.role),
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

pub(super) fn run_top_app_bar_case(case: &Case, theme: &Theme) {
    let variant = top_app_bar_variant(&case.input.variant, &case.id);
    let scrolled = case.input.scrolled;

    for assertion in &case.assertions {
        match assertion.kind.as_str() {
            "color" => assert_color_close(
                &case.id,
                &assertion.role,
                actual_top_app_bar_color(theme, variant, scrolled, &assertion.role),
                token_color(theme, require_token(assertion, "token")),
            ),
            "metric" => assert_px_eq(
                &case.id,
                &assertion.role,
                actual_top_app_bar_metric(theme, variant, scrolled, &assertion.role),
                token_metric(theme, require_token(assertion, "token")),
            ),
            "corners_metric" => assert_corners_eq(
                &case.id,
                &assertion.role,
                actual_top_app_bar_corners(theme, variant, &assertion.role),
                Corners::all(token_metric(theme, require_token(assertion, "token"))),
            ),
            "text_style_source" => assert_text_style_eq(
                &case.id,
                &assertion.role,
                actual_top_app_bar_text_style(theme, variant, &assertion.role),
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
