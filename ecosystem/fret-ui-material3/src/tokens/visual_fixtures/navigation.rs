use super::*;
use crate::navigation_drawer::NavigationDrawerVariant;
use crate::tokens::{navigation_bar, navigation_drawer, navigation_rail, tabs, top_app_bar};
use crate::top_app_bar::TopAppBarVariant;

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

// Navigation-family token outcome helpers live with the cases that exercise them.
fn actual_tabs_color(
    theme: &Theme,
    active: bool,
    interaction: tabs::TabInteraction,
    role: &str,
) -> Color {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "container_color" => tabs::container_background_for(theme, kind),
        "active_indicator_color" => tabs::active_indicator_color(theme),
        "label_color" => tabs::label_color_for(theme, kind, active, interaction),
        "state_layer_color" => tabs::state_layer_color_for(theme, kind, active, interaction),
        other => panic!("unsupported tabs color role {other}"),
    }
}

fn actual_tabs_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "container_height" => tabs::container_height_for(theme, kind),
        "active_indicator_height" => tabs::active_indicator_height(theme),
        "active_indicator_min_width" => tabs::active_indicator_min_width(theme),
        "scrollable_edge_padding" if variant == "scrollable" => {
            tabs::scrollable_edge_padding_for(theme, kind)
        }
        "scrollable_min_tab_width" if variant == "scrollable" => {
            tabs::scrollable_min_tab_width_for(theme, kind)
        }
        other => panic!("unsupported tabs metric role {other}"),
    }
}

fn actual_tabs_number(
    theme: &Theme,
    active: bool,
    interaction: tabs::TabInteraction,
    role: &str,
) -> f32 {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "state_layer_opacity" => tabs::state_layer_opacity_for(theme, kind, active, interaction),
        "pressed_state_layer_opacity" => tabs::pressed_state_layer_opacity_for(theme, kind, active),
        other => panic!("unsupported tabs number role {other}"),
    }
}

fn actual_tabs_corners(theme: &Theme, role: &str) -> Corners {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "active_indicator_shape" => tabs::active_indicator_shape_for(theme, kind),
        other => panic!("unsupported tabs corners role {other}"),
    }
}

fn actual_tabs_text_style(theme: &Theme, role: &str) -> TextStyle {
    let kind = tabs::NavigationTabKind::Primary;
    match role {
        "label_text_style" => tabs::label_text_style_for(theme, kind),
        other => panic!("unsupported tabs text style role {other}"),
    }
}

fn actual_navigation_bar_color(
    theme: &Theme,
    active: bool,
    interaction: navigation_bar::NavigationBarItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_bar::container_background(theme),
        "container_shadow_color" => navigation_bar::container_shadow_color(theme),
        "active_indicator_color" => navigation_bar::active_indicator_color(theme),
        "label_color" => navigation_bar::label_color(theme, active, interaction),
        "icon_color" => navigation_bar::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_bar::state_layer_color(theme, active, interaction),
        other => panic!("unsupported navigation bar color role {other}"),
    }
}

fn actual_navigation_bar_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_height" => navigation_bar::container_height(theme),
        "container_elevation" => navigation_bar::container_elevation(theme),
        "active_indicator_width" => navigation_bar::active_indicator_width(theme),
        "active_indicator_height" => navigation_bar::active_indicator_height(theme),
        "active_indicator_top_offset" => navigation_bar::active_indicator_top_offset(theme),
        "icon_size" => navigation_bar::icon_size(theme),
        "item_gap" => navigation_bar::item_gap(theme),
        other => panic!("unsupported navigation bar metric role {other}"),
    }
}

fn actual_navigation_bar_number(
    theme: &Theme,
    interaction: navigation_bar::NavigationBarItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => navigation_bar::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => navigation_bar::pressed_state_layer_opacity(theme),
        other => panic!("unsupported navigation bar number role {other}"),
    }
}

fn actual_navigation_bar_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_bar::container_shape(theme),
        "active_indicator_shape" => navigation_bar::active_indicator_shape(theme),
        other => panic!("unsupported navigation bar corners role {other}"),
    }
}

fn actual_navigation_bar_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_bar::label_text_style(theme, active),
        other => panic!("unsupported navigation bar text style role {other}"),
    }
}

fn actual_navigation_rail_color(
    theme: &Theme,
    active: bool,
    interaction: navigation_rail::NavigationRailItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_rail::container_background(theme),
        "active_indicator_color" => navigation_rail::active_indicator_color(theme),
        "label_color" => navigation_rail::label_color(theme, active, interaction),
        "icon_color" => navigation_rail::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_rail::state_layer_color(theme, active, interaction),
        other => panic!("unsupported navigation rail color role {other}"),
    }
}

fn actual_navigation_rail_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    let has_label = variant != "no_label";
    match role {
        "container_width" => navigation_rail::container_width(theme),
        "item_width" => navigation_rail::item_width(theme),
        "item_height" => navigation_rail::item_height(theme),
        "vertical_padding" => navigation_rail::vertical_padding(theme),
        "active_indicator_width" => navigation_rail::active_indicator_width(theme),
        "active_indicator_height" => navigation_rail::active_indicator_height(theme, has_label),
        "icon_size" => navigation_rail::icon_size(theme),
        other => panic!("unsupported navigation rail metric role {other}"),
    }
}

fn actual_navigation_rail_number(
    theme: &Theme,
    interaction: navigation_rail::NavigationRailItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => navigation_rail::state_layer_opacity(theme, interaction),
        "pressed_state_layer_opacity" => navigation_rail::pressed_state_layer_opacity(theme),
        other => panic!("unsupported navigation rail number role {other}"),
    }
}

fn actual_navigation_rail_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_rail::container_shape(theme),
        "active_indicator_shape" => navigation_rail::active_indicator_shape(theme),
        other => panic!("unsupported navigation rail corners role {other}"),
    }
}

fn actual_navigation_rail_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_rail::label_text_style(theme, active),
        other => panic!("unsupported navigation rail text style role {other}"),
    }
}

fn actual_navigation_drawer_color(
    theme: &Theme,
    variant: NavigationDrawerVariant,
    active: bool,
    interaction: navigation_drawer::NavigationDrawerItemInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => navigation_drawer::container_background(theme, variant),
        "active_indicator_color" => navigation_drawer::active_indicator_color(theme),
        "label_color" => navigation_drawer::label_color(theme, active, interaction),
        "icon_color" => navigation_drawer::icon_color(theme, active, interaction),
        "state_layer_color" => navigation_drawer::state_layer_color(theme, active, interaction),
        "large_badge_label_color" => navigation_drawer::large_badge_label_color(theme),
        "scrim_color" => navigation_drawer::scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            navigation_drawer::scrim_color(theme),
            navigation_drawer::scrim_opacity(theme),
        ),
        other => panic!("unsupported navigation drawer color role {other}"),
    }
}

fn actual_navigation_drawer_metric(
    theme: &Theme,
    variant: NavigationDrawerVariant,
    role: &str,
) -> Px {
    match role {
        "container_width" => navigation_drawer::container_width(theme),
        "container_elevation" => navigation_drawer::container_elevation(theme, variant),
        "item_horizontal_padding" => navigation_drawer::item_horizontal_padding(theme),
        "active_indicator_width" => navigation_drawer::active_indicator_width(theme),
        "active_indicator_height" => navigation_drawer::active_indicator_height(theme),
        "icon_size" => navigation_drawer::icon_size(theme),
        other => panic!("unsupported navigation drawer metric role {other}"),
    }
}

fn actual_navigation_drawer_number(
    theme: &Theme,
    interaction: navigation_drawer::NavigationDrawerItemInteraction,
    role: &str,
) -> f32 {
    match role {
        "state_layer_opacity" => {
            navigation_drawer::state_layer_target_opacity(theme, true, interaction)
        }
        "pressed_state_layer_opacity" => navigation_drawer::pressed_state_layer_opacity(theme),
        "scrim_opacity" => navigation_drawer::scrim_opacity(theme),
        other => panic!("unsupported navigation drawer number role {other}"),
    }
}

fn actual_navigation_drawer_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => navigation_drawer::container_shape(theme),
        "active_indicator_shape" => navigation_drawer::active_indicator_shape(theme),
        other => panic!("unsupported navigation drawer corners role {other}"),
    }
}

fn actual_navigation_drawer_text_style(theme: &Theme, active: bool, role: &str) -> TextStyle {
    match role {
        "label_text_style" => navigation_drawer::label_text_style(theme, active),
        "large_badge_label_text_style" => navigation_drawer::large_badge_label_text_style(theme),
        other => panic!("unsupported navigation drawer text style role {other}"),
    }
}

fn actual_top_app_bar_color(
    theme: &Theme,
    variant: TopAppBarVariant,
    scrolled: bool,
    role: &str,
) -> Color {
    match role {
        "container_color" => top_app_bar::container_background(theme, variant, scrolled),
        "headline_color" => top_app_bar::headline_color(theme, variant),
        "leading_icon_color" => top_app_bar::leading_icon_color(theme, variant),
        "trailing_icon_color" => top_app_bar::trailing_icon_color(theme, variant),
        other => panic!("unsupported top app bar color role {other}"),
    }
}

fn actual_top_app_bar_metric(
    theme: &Theme,
    variant: TopAppBarVariant,
    scrolled: bool,
    role: &str,
) -> Px {
    match role {
        "container_height" => top_app_bar::container_height(theme, variant),
        "container_elevation" => top_app_bar::container_elevation(theme, variant, scrolled),
        other => panic!("unsupported top app bar metric role {other}"),
    }
}

fn actual_top_app_bar_corners(theme: &Theme, variant: TopAppBarVariant, role: &str) -> Corners {
    match role {
        "container_shape" => top_app_bar::container_shape(theme, variant),
        other => panic!("unsupported top app bar corners role {other}"),
    }
}

fn actual_top_app_bar_text_style(
    theme: &Theme,
    variant: TopAppBarVariant,
    role: &str,
) -> TextStyle {
    match role {
        "headline_text_style" => top_app_bar::headline_text_style(theme, variant),
        other => panic!("unsupported top app bar text style role {other}"),
    }
}

fn tab_interaction(value: Option<&str>, case_id: &str) -> tabs::TabInteraction {
    match value.unwrap_or("none") {
        "none" => tabs::TabInteraction::Default,
        "hovered" => tabs::TabInteraction::Hovered,
        "focused" => tabs::TabInteraction::Focused,
        "pressed" => tabs::TabInteraction::Pressed,
        other => panic!("{case_id}: unsupported tabs interaction {other}"),
    }
}

fn navigation_bar_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_bar::NavigationBarItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_bar::NavigationBarItemInteraction::Default,
        "hovered" => navigation_bar::NavigationBarItemInteraction::Hovered,
        "focused" => navigation_bar::NavigationBarItemInteraction::Focused,
        "pressed" => navigation_bar::NavigationBarItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation bar interaction {other}"),
    }
}

fn navigation_rail_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_rail::NavigationRailItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_rail::NavigationRailItemInteraction::Default,
        "hovered" => navigation_rail::NavigationRailItemInteraction::Hovered,
        "focused" => navigation_rail::NavigationRailItemInteraction::Focused,
        "pressed" => navigation_rail::NavigationRailItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation rail interaction {other}"),
    }
}

fn navigation_drawer_interaction(
    value: Option<&str>,
    case_id: &str,
) -> navigation_drawer::NavigationDrawerItemInteraction {
    match value.unwrap_or("none") {
        "none" => navigation_drawer::NavigationDrawerItemInteraction::Default,
        "hovered" => navigation_drawer::NavigationDrawerItemInteraction::Hovered,
        "focused" => navigation_drawer::NavigationDrawerItemInteraction::Focused,
        "pressed" => navigation_drawer::NavigationDrawerItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported navigation drawer interaction {other}"),
    }
}

fn navigation_drawer_variant(value: &str, case_id: &str) -> NavigationDrawerVariant {
    match value {
        "standard" => NavigationDrawerVariant::Standard,
        "modal_navigation_drawer" => NavigationDrawerVariant::Modal,
        "modal" | "modal_content" => NavigationDrawerVariant::Modal,
        other => panic!("{case_id}: unsupported navigation drawer variant {other}"),
    }
}

fn top_app_bar_variant(value: &str, case_id: &str) -> TopAppBarVariant {
    match value {
        "small" => TopAppBarVariant::Small,
        "small_centered" | "center_aligned" => TopAppBarVariant::SmallCentered,
        "medium" => TopAppBarVariant::Medium,
        "large" => TopAppBarVariant::Large,
        other => panic!("{case_id}: unsupported top app bar variant {other}"),
    }
}
