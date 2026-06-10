use super::visual_fixture_model::{Component, SchemeModeFixture, load_suite, theme_for};

mod assertions;
mod fields;
mod input;
mod navigation;
mod overlays;
mod selection;
mod surfaces;
mod token_lookup;
mod typography_helpers;

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
