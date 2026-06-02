use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::interaction::PressableInteraction;
use crate::foundation::token_resolver::blend_over;
use crate::tokens::typography as token_typography;

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
