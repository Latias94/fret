use super::*;
use crate::tokens::{dialog, dropdown_menu, menu, sheet_bottom, snackbar, tooltip};

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

// Overlay-family token outcome helpers live with the cases that exercise them.
fn actual_menu_color(
    theme: &Theme,
    enabled: bool,
    interaction: menu::MenuItemInteraction,
    role: &str,
) -> Color {
    let (label, state_layer, _) = menu::item_outcomes(theme, enabled, interaction);
    match role {
        "container_color" => menu::container_background(theme),
        "container_shadow_color" => menu::container_shadow_color(theme),
        "divider_color" => menu::divider_color(theme),
        "item_label_color" | "label_color" => label,
        "item_icon_color" => menu::item_icon_color(theme, enabled, interaction),
        "item_supporting_text_color" => menu::item_supporting_text_color(theme, enabled),
        "item_trailing_text_color" => menu::item_trailing_text_color(theme, enabled),
        "section_label_color" => menu::section_label_color(theme),
        "state_layer_color" => state_layer,
        other => panic!("unsupported menu color role {other}"),
    }
}

fn actual_menu_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "item_height" => menu::list_item_height(theme),
        "item_two_line_height" => menu::list_item_two_line_height(theme),
        "item_min_width" => menu::item_min_width(theme),
        "item_max_width" => menu::item_max_width(theme),
        "container_vertical_padding" => menu::container_vertical_padding(theme),
        "item_horizontal_padding" => menu::item_horizontal_padding(theme),
        "item_slot_gap" => menu::item_slot_gap(theme),
        "item_icon_size" => menu::item_icon_size(theme),
        "section_label_height" => menu::section_label_height(theme),
        "container_elevation" => menu::container_elevation(theme),
        "divider_height" => menu::divider_height(theme),
        other => panic!("unsupported menu metric role {other}"),
    }
}

fn actual_dropdown_menu_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "divider_margin_total" => dropdown_menu::divider_margin_total(theme),
        "collision_padding" => dropdown_menu::collision_padding(theme).left,
        "max_height" => dropdown_menu::max_height(theme),
        other => actual_menu_metric(theme, other),
    }
}

fn actual_menu_number(
    theme: &Theme,
    enabled: bool,
    interaction: menu::MenuItemInteraction,
    role: &str,
) -> f32 {
    let (_, _, state_layer_opacity) = menu::item_outcomes(theme, enabled, interaction);
    match role {
        "state_layer_opacity" => state_layer_opacity,
        "pressed_state_layer_opacity" => menu::pressed_state_layer_opacity(theme),
        other => panic!("unsupported menu number role {other}"),
    }
}

fn actual_menu_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => menu::container_shape(theme),
        other => panic!("unsupported menu corners role {other}"),
    }
}

fn actual_menu_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "item_label_text_style" | "label_text_style" => menu::item_label_text_style(theme),
        "item_supporting_text_style" => menu::item_supporting_text_style(theme),
        "item_trailing_text_style" => menu::item_trailing_text_style(theme),
        "section_label_text_style" => menu::section_label_text_style(theme),
        other => panic!("unsupported menu text style role {other}"),
    }
}

fn actual_dialog_color(
    theme: &Theme,
    interaction: dialog::DialogActionInteraction,
    role: &str,
) -> Color {
    match role {
        "scrim_color" => dialog::scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            dialog::scrim_color(theme),
            dialog::scrim_opacity(theme, 0.32),
        ),
        "container_color" => dialog::container_background(theme),
        "container_shadow_color" => dialog::container_shadow_color(theme),
        "headline_color" => dialog::headline_color(theme),
        "supporting_text_color" => dialog::supporting_text_color(theme),
        "action_label_color" => dialog::action_label_color(theme, interaction),
        "action_state_layer_color" => dialog::action_state_layer_color(theme, interaction),
        other => panic!("unsupported dialog color role {other}"),
    }
}

fn actual_dialog_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "container_elevation" => dialog::container_elevation(theme),
        "action_height" => dialog::action_height(theme),
        "container_min_width" => dialog::container_min_width(theme),
        "container_max_width" => dialog::container_max_width(theme),
        other => panic!("unsupported dialog metric role {other}"),
    }
}

fn actual_dialog_number(
    theme: &Theme,
    interaction: dialog::DialogActionInteraction,
    role: &str,
) -> f32 {
    match role {
        "scrim_opacity" => dialog::scrim_opacity(theme, 0.32),
        "action_state_layer_opacity" => {
            dialog::action_state_layer_target_opacity(theme, interaction)
        }
        "action_pressed_state_layer_opacity" => dialog::action_pressed_state_layer_opacity(theme),
        other => panic!("unsupported dialog number role {other}"),
    }
}

fn actual_dialog_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => dialog::container_shape(theme),
        "action_corner_radii" => dialog::action_corner_radii(theme),
        other => panic!("unsupported dialog corners role {other}"),
    }
}

fn actual_dialog_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "headline_text_style" => dialog::headline_text_style(theme),
        "supporting_text_style" => dialog::supporting_text_style(theme),
        "action_label_text_style" => dialog::action_label_text_style(theme),
        other => panic!("unsupported dialog text style role {other}"),
    }
}

fn actual_bottom_sheet_color(theme: &Theme, role: &str) -> Color {
    match role {
        "scrim_color" => sheet_bottom::modal_scrim_color(theme),
        "scrim_color_alpha" => alpha_color(
            sheet_bottom::modal_scrim_color(theme),
            sheet_bottom::modal_scrim_opacity(theme, 0.32),
        ),
        "container_color" => sheet_bottom::docked_container_color(theme),
        "drag_handle_color" => sheet_bottom::docked_drag_handle_color(theme),
        "drag_handle_color_alpha" => alpha_color(
            sheet_bottom::docked_drag_handle_color(theme),
            sheet_bottom::docked_drag_handle_opacity(theme),
        ),
        "focus_indicator_color" => sheet_bottom::focus_indicator_color(theme),
        other => panic!("unsupported bottom sheet color role {other}"),
    }
}

fn actual_bottom_sheet_metric(theme: &Theme, role: &str) -> Px {
    match role {
        "modal_container_elevation" => sheet_bottom::docked_modal_elevation(theme),
        "standard_container_elevation" => sheet_bottom::docked_standard_elevation(theme),
        "drag_handle_width" => sheet_bottom::docked_drag_handle_width(theme),
        "drag_handle_height" => sheet_bottom::docked_drag_handle_height(theme),
        "focus_indicator_thickness" => sheet_bottom::focus_indicator_thickness(theme),
        "focus_indicator_outline_offset" => sheet_bottom::focus_indicator_outline_offset(theme),
        other => panic!("unsupported bottom sheet metric role {other}"),
    }
}

fn actual_bottom_sheet_number(theme: &Theme, role: &str) -> f32 {
    match role {
        "scrim_opacity" => sheet_bottom::modal_scrim_opacity(theme, 0.32),
        "drag_handle_opacity" => sheet_bottom::docked_drag_handle_opacity(theme),
        other => panic!("unsupported bottom sheet number role {other}"),
    }
}

fn actual_bottom_sheet_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => sheet_bottom::docked_container_shape(theme),
        other => panic!("unsupported bottom sheet corners role {other}"),
    }
}

fn actual_tooltip_color(theme: &Theme, variant: &str, role: &str) -> Color {
    match (variant, role) {
        ("plain", "container_color") => tooltip::plain_container_background(theme),
        ("plain", "supporting_text_color") => tooltip::plain_supporting_text_color(theme),
        ("rich", "container_color") => tooltip::rich_container_background(theme),
        ("rich", "container_shadow_color") => tooltip::rich_container_shadow_color(theme),
        ("rich", "subhead_color") => tooltip::rich_subhead_color(theme),
        ("rich", "supporting_text_color") => tooltip::rich_supporting_text_color(theme),
        (_, "shadow_color") => tooltip::shadow_color(theme),
        _ => panic!("unsupported tooltip color role {variant}:{role}"),
    }
}

fn actual_tooltip_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        ("plain", "container_max_width") => tooltip::plain_container_max_width(theme),
        ("rich", "container_max_width") => tooltip::rich_container_max_width(theme),
        ("rich", "container_elevation") => tooltip::rich_container_elevation(theme),
        (_, "container_min_width") => tooltip::container_min_width(theme),
        (_, "container_min_height") => tooltip::container_min_height(theme),
        _ => panic!("unsupported tooltip metric role {variant}:{role}"),
    }
}

fn actual_tooltip_corners(theme: &Theme, variant: &str, role: &str) -> Corners {
    match (variant, role) {
        ("plain", "container_shape") => tooltip::plain_container_shape(theme),
        ("rich", "container_shape") => tooltip::rich_container_shape(theme),
        _ => panic!("unsupported tooltip corners role {variant}:{role}"),
    }
}

fn actual_tooltip_text_style(theme: &Theme, variant: &str, role: &str) -> TextStyle {
    match (variant, role) {
        ("plain", "supporting_text_style") => tooltip::plain_supporting_text_style(theme),
        ("rich", "subhead_text_style") => tooltip::rich_subhead_text_style(theme),
        ("rich", "supporting_text_style") => tooltip::rich_supporting_text_style(theme),
        _ => panic!("unsupported tooltip text style role {variant}:{role}"),
    }
}

fn actual_snackbar_color(
    theme: &Theme,
    interaction: snackbar::SnackbarActionInteraction,
    role: &str,
) -> Color {
    match role {
        "container_color" => snackbar::container_background(theme),
        "container_shadow_color" => snackbar::container_shadow_color(theme),
        "supporting_text_color" => snackbar::supporting_text_color(theme),
        "action_label_color" => snackbar::action_label_color(theme, interaction),
        "action_state_layer_color" => snackbar::action_state_layer_color(theme, interaction),
        "icon_color" => snackbar::icon_color(theme, interaction),
        "icon_state_layer_color" => snackbar::icon_state_layer_color(theme, interaction),
        other => panic!("unsupported snackbar color role {other}"),
    }
}

fn actual_snackbar_metric(theme: &Theme, variant: &str, role: &str) -> Px {
    match (variant, role) {
        (_, "icon_size") => snackbar::icon_size(theme),
        (_, "container_elevation") => snackbar::container_elevation(theme),
        ("single_line", "container_height") => snackbar::single_line_min_height(theme)
            .unwrap_or_else(|| panic!("expected single line snackbar height")),
        ("two_line", "container_height") => snackbar::two_line_min_height(theme)
            .unwrap_or_else(|| panic!("expected two line snackbar height")),
        _ => panic!("unsupported snackbar metric role {variant}:{role}"),
    }
}

fn actual_snackbar_number(
    theme: &Theme,
    interaction: snackbar::SnackbarActionInteraction,
    role: &str,
) -> f32 {
    match role {
        "action_state_layer_opacity" => snackbar::action_state_layer_opacity(theme, interaction),
        "icon_state_layer_opacity" => snackbar::icon_state_layer_opacity(theme, interaction),
        other => panic!("unsupported snackbar number role {other}"),
    }
}

fn actual_snackbar_corners(theme: &Theme, role: &str) -> Corners {
    match role {
        "container_shape" => snackbar::container_shape(theme),
        other => panic!("unsupported snackbar corners role {other}"),
    }
}

fn actual_snackbar_text_style(theme: &Theme, role: &str) -> TextStyle {
    match role {
        "supporting_text_style" => snackbar::supporting_text_style(theme),
        "action_label_text_style" => snackbar::action_label_text_style(theme),
        other => panic!("unsupported snackbar text style role {other}"),
    }
}

fn menu_interaction(value: Option<&str>, case_id: &str) -> menu::MenuItemInteraction {
    match value.unwrap_or("none") {
        "none" => menu::MenuItemInteraction::Default,
        "hovered" => menu::MenuItemInteraction::Hovered,
        "focused" => menu::MenuItemInteraction::Focused,
        "pressed" => menu::MenuItemInteraction::Pressed,
        other => panic!("{case_id}: unsupported menu interaction {other}"),
    }
}

fn dialog_action_interaction(
    value: Option<&str>,
    case_id: &str,
) -> dialog::DialogActionInteraction {
    match value.unwrap_or("none") {
        "none" => dialog::DialogActionInteraction::Default,
        "hovered" => dialog::DialogActionInteraction::Hovered,
        "focused" => dialog::DialogActionInteraction::Focused,
        "pressed" => dialog::DialogActionInteraction::Pressed,
        other => panic!("{case_id}: unsupported dialog action interaction {other}"),
    }
}

fn snackbar_interaction(value: Option<&str>, case_id: &str) -> snackbar::SnackbarActionInteraction {
    match value.unwrap_or("none") {
        "none" => snackbar::SnackbarActionInteraction::Default,
        "hovered" => snackbar::SnackbarActionInteraction::Hovered,
        "focused" => snackbar::SnackbarActionInteraction::Focused,
        "pressed" => snackbar::SnackbarActionInteraction::Pressed,
        other => panic!("{case_id}: unsupported snackbar interaction {other}"),
    }
}
