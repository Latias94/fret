use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::blend_over;
use crate::tokens::{
    dialog, dropdown_menu, menu, sheet_bottom, snackbar, tooltip, typography as token_typography,
};

use super::visual_fixture_model::{
    Assertion, Case, Component, SchemeModeFixture, load_suite, theme_for,
};

mod fields;
mod navigation;
mod overlays;
mod selection;
mod surfaces;

#[test]
fn material3_token_visual_fixtures_match_expected_token_outcomes() {
    let suite = load_suite();
    assert_eq!(suite.schema_version, 1);

    for case in &suite.cases {
        let theme = theme_for(&case.scheme);
        match case.component {
            Component::Autocomplete => fields::run_autocomplete_case(case, &theme),
            Component::Badge => surfaces::run_badge_case(case, &theme),
            Component::BottomSheet => overlays::run_bottom_sheet_case(case, &theme),
            Component::Button => surfaces::run_button_case(case, &theme),
            Component::Card => surfaces::run_card_case(case, &theme),
            Component::CarouselItem => surfaces::run_carousel_item_case(case, &theme),
            Component::Checkbox => selection::run_checkbox_case(case, &theme),
            Component::Chip => selection::run_chip_case(case, &theme),
            Component::DatePicker => fields::run_date_picker_case(case, &theme),
            Component::Dialog => overlays::run_dialog_case(case, &theme),
            Component::Divider => surfaces::run_divider_case(case, &theme),
            Component::DropdownMenu => overlays::run_dropdown_menu_case(case, &theme),
            Component::ExposedDropdown => fields::run_autocomplete_case(case, &theme),
            Component::Fab => surfaces::run_fab_case(case, &theme),
            Component::FilterChip => selection::run_filter_chip_case(case, &theme),
            Component::IconButton => selection::run_icon_button_case(case, &theme),
            Component::InputChip => selection::run_input_chip_case(case, &theme),
            Component::List => surfaces::run_list_case(case, &theme),
            Component::Menu => overlays::run_menu_case(case, &theme),
            Component::ModalNavigationDrawer => {
                navigation::run_navigation_drawer_case(case, &theme)
            }
            Component::NavigationBar => navigation::run_navigation_bar_case(case, &theme),
            Component::NavigationDrawer => navigation::run_navigation_drawer_case(case, &theme),
            Component::NavigationRail => navigation::run_navigation_rail_case(case, &theme),
            Component::ProgressIndicator => surfaces::run_progress_indicator_case(case, &theme),
            Component::Radio => selection::run_radio_case(case, &theme),
            Component::SearchBar => fields::run_search_bar_case(case, &theme),
            Component::SearchView => fields::run_search_view_case(case, &theme),
            Component::SegmentedButton => selection::run_segmented_button_case(case, &theme),
            Component::Select => fields::run_select_case(case, &theme),
            Component::Slider => selection::run_slider_case(case, &theme),
            Component::Snackbar => overlays::run_snackbar_case(case, &theme),
            Component::SuggestionChip => selection::run_suggestion_chip_case(case, &theme),
            Component::Switch => selection::run_switch_case(case, &theme),
            Component::Tabs => navigation::run_tabs_case(case, &theme),
            Component::TextField => fields::run_text_field_case(case, &theme),
            Component::TimePicker => fields::run_time_picker_case(case, &theme),
            Component::Tooltip => overlays::run_tooltip_case(case, &theme),
            Component::TopAppBar => navigation::run_top_app_bar_case(case, &theme),
        }
    }
}

#[test]
fn material3_token_visual_fixtures_cover_light_and_dark_scheme_modes() {
    let suite = load_suite();

    assert!(
        suite
            .cases
            .iter()
            .any(|case| matches!(case.scheme.mode, SchemeModeFixture::Light)),
        "visual fixture matrix must cover at least one light scheme case"
    );
    assert!(
        suite
            .cases
            .iter()
            .any(|case| matches!(case.scheme.mode, SchemeModeFixture::Dark)),
        "visual fixture matrix must cover at least one dark scheme case"
    );
}

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

fn enabled_input(case: &Case) -> bool {
    case.input.enabled.unwrap_or(!case.input.disabled)
}

fn pressable_interaction(value: Option<&str>, case_id: &str) -> Option<PressableInteraction> {
    match value.unwrap_or("none") {
        "none" => None,
        "hovered" => Some(PressableInteraction::Hovered),
        "focused" => Some(PressableInteraction::Focused),
        "pressed" => Some(PressableInteraction::Pressed),
        other => panic!("{case_id}: unsupported pressable interaction {other}"),
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

fn require_token<'a>(assertion: &'a Assertion, field: &str) -> &'a str {
    let token = match field {
        "token" => assertion.token.as_deref(),
        "source_token" => assertion.source_token.as_deref(),
        "color_token" => assertion.color_token.as_deref(),
        "opacity_token" => assertion.opacity_token.as_deref(),
        "base_color_token" => assertion.base_color_token.as_deref(),
        "overlay_color_token" => assertion.overlay_color_token.as_deref(),
        other => panic!("unsupported token field {other}"),
    };
    token.unwrap_or_else(|| panic!("{} missing {field}", assertion.role))
}

fn require_value(assertion: &Assertion) -> f32 {
    assertion
        .value
        .unwrap_or_else(|| panic!("{} missing value", assertion.role))
}

fn token_color(theme: &Theme, key: &str) -> Color {
    theme
        .color_by_key(key)
        .unwrap_or_else(|| panic!("expected color token {key}"))
}

fn token_metric(theme: &Theme, key: &str) -> Px {
    theme
        .metric_by_key(key)
        .unwrap_or_else(|| panic!("expected metric token {key}"))
}

fn token_number(theme: &Theme, key: &str) -> f32 {
    theme
        .number_by_key(key)
        .unwrap_or_else(|| panic!("expected number token {key}"))
}

fn token_corners(theme: &Theme, key: &str) -> Corners {
    theme
        .corners_by_key(key)
        .unwrap_or_else(|| panic!("expected corners token {key}"))
}

fn token_text_style(theme: &Theme, key: &str) -> TextStyle {
    theme
        .text_style_by_key(key)
        .unwrap_or_else(|| panic!("expected text style token {key}"))
}

fn color_with_alpha(theme: &Theme, color_token: &str, opacity_token: &str) -> Color {
    let mut color = token_color(theme, color_token);
    color.a = (color.a * token_number(theme, opacity_token)).clamp(0.0, 1.0);
    color
}

fn alpha_color(mut color: Color, opacity: f32) -> Color {
    color.a = (color.a * opacity).clamp(0.0, 1.0);
    color
}

fn control_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Control)
}

fn control_text_style_with_weight(theme: &Theme, source_key: &str, weight_key: &str) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Control)
}

fn content_text_style_with_weight(theme: &Theme, source_key: &str, weight_key: &str) -> TextStyle {
    text_style_with_weight(theme, source_key, weight_key, TextIntent::Content)
}

fn text_style_with_weight(
    theme: &Theme,
    source_key: &str,
    weight_key: &str,
    intent: TextIntent,
) -> TextStyle {
    let _ = token_number(theme, weight_key);
    token_typography::text_style_with_weight(theme, None, source_key, Some(weight_key), intent)
}

fn text_intent_for_role(role: &str) -> TextIntent {
    if role.contains("action") || role.contains("label") {
        TextIntent::Control
    } else {
        TextIntent::Content
    }
}

fn content_text_style(theme: &Theme, key: &str) -> TextStyle {
    typography::with_intent(token_text_style(theme, key), TextIntent::Content)
}

fn assert_text_style_alias(theme: &Theme, case: &Case, assertion: &Assertion) {
    assert_eq!(
        token_text_style(theme, require_token(assertion, "token")),
        token_text_style(theme, require_token(assertion, "source_token")),
        "{}:{} text style alias mismatch",
        case.id,
        assertion.role
    );
}

fn assert_text_style_eq(case_id: &str, role: &str, actual: TextStyle, expected: TextStyle) {
    assert_eq!(
        actual, expected,
        "{case_id}:{role} text style mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_color_close(case_id: &str, role: &str, actual: Color, expected: Color) {
    assert!(
        close(actual.r, expected.r)
            && close(actual.g, expected.g)
            && close(actual.b, expected.b)
            && close(actual.a, expected.a),
        "{case_id}:{role} color mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_px_eq(case_id: &str, role: &str, actual: Px, expected: Px) {
    assert!(
        close(actual.0, expected.0),
        "{case_id}:{role} px mismatch: actual={actual:?} expected={expected:?}"
    );
}

fn assert_number_close(case_id: &str, role: &str, actual: f32, expected: f32) {
    assert!(
        close(actual, expected),
        "{case_id}:{role} number mismatch: actual={actual} expected={expected}"
    );
}

fn assert_corners_eq(case_id: &str, role: &str, actual: Corners, expected: Corners) {
    assert_px_eq(
        case_id,
        &format!("{role}.top_left"),
        actual.top_left,
        expected.top_left,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.top_right"),
        actual.top_right,
        expected.top_right,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.bottom_right"),
        actual.bottom_right,
        expected.bottom_right,
    );
    assert_px_eq(
        case_id,
        &format!("{role}.bottom_left"),
        actual.bottom_left,
        expected.bottom_left,
    );
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() <= 0.0001
}
