mod support;

// Broad Material3 headless golden suites live here so focused component tests do not
// inherit unrelated golden refresh churn. Keep Radio-specific behavior in radio_alignment.rs.

#[test]
fn material3_headless_controls_suite_goldens_v1() {
    support::headless_golden_runners::controls::run_material3_headless_controls_suite_goldens_v1();
}

#[test]
fn material3_headless_fab_suite_goldens_v1() {
    support::headless_golden_runners::fab::run_material3_headless_fab_suite_goldens_v1();
}

#[test]
fn material3_headless_icon_button_suite_goldens_v1() {
    support::headless_golden_runners::icon_button::run_material3_headless_icon_button_suite_goldens_v1();
}

#[test]
fn material3_headless_segmented_button_suite_goldens_v1() {
    support::headless_golden_runners::segmented_button::run_material3_headless_segmented_button_suite_goldens_v1();
}

#[test]
fn material3_headless_radio_suite_goldens_v1() {
    support::headless_golden_runners::radio::run_material3_headless_radio_suite_goldens_v1();
}

#[test]
fn material3_headless_badge_suite_goldens_v1() {
    support::headless_golden_runners::badge::run_material3_headless_badge_suite_goldens_v1();
}

#[test]
fn material3_headless_top_app_bar_suite_goldens_v1() {
    support::headless_golden_runners::top_app_bar::run_material3_headless_top_app_bar_suite_goldens_v1();
}

#[test]
fn material3_headless_navigation_suite_goldens_v1() {
    support::headless_golden_runners::navigation::run_material3_headless_navigation_suite_goldens_v1(
    );
}

#[test]
fn material3_headless_snackbar_suite_goldens_v1() {
    support::headless_golden_runners::snackbar::run_material3_headless_snackbar_suite_goldens_v1();
}

#[test]
fn material3_headless_divider_suite_goldens_v1() {
    support::headless_golden_runners::divider::run_material3_headless_divider_suite_goldens_v1();
}

#[test]
fn material3_headless_list_suite_goldens_v1() {
    support::headless_golden_runners::list::run_material3_headless_list_suite_goldens_v1();
}

#[test]
fn material3_headless_progress_indicator_suite_goldens_v1() {
    support::headless_golden_runners::progress_indicator::run_material3_headless_progress_indicator_suite_goldens_v1();
}

#[test]
fn material3_headless_slider_suite_goldens_v1() {
    support::headless_golden_runners::slider::run_material3_headless_slider_suite_goldens_v1();
}

#[test]
fn material3_headless_overlays_suite_goldens_v1() {
    support::headless_golden_runners::overlays::run_material3_headless_overlays_suite_goldens_v1();
}

#[test]
fn material3_headless_autocomplete_suite_goldens_v1() {
    support::headless_golden_runners::autocomplete::run_material3_headless_autocomplete_suite_goldens_v1();
}

#[test]
fn material3_headless_exposed_dropdown_suite_goldens_v1() {
    support::headless_golden_runners::exposed_dropdown::run_material3_headless_exposed_dropdown_suite_goldens_v1();
}

#[test]
fn material3_headless_menu_dialog_style_suite_goldens_v1() {
    support::headless_golden_runners::menu_dialog_style::run_material3_headless_menu_dialog_style_suite_goldens_v1();
}

#[test]
fn material3_headless_bottom_sheet_suite_goldens_v1() {
    support::headless_golden_runners::bottom_sheet::run_material3_headless_bottom_sheet_suite_goldens_v1();
}

#[test]
fn material3_headless_date_picker_suite_goldens_v1() {
    support::headless_golden_runners::date_picker::run_material3_headless_date_picker_suite_goldens_v1();
}

#[test]
fn material3_headless_time_picker_suite_goldens_v1() {
    support::headless_golden_runners::time_picker::run_material3_headless_time_picker_suite_goldens_v1();
}

#[test]
fn material3_headless_text_field_suite_goldens_v1() {
    support::headless_golden_runners::text_field::run_material3_headless_text_field_suite_goldens_v1();
}

#[test]
fn material3_headless_search_bar_suite_goldens_v1() {
    support::headless_golden_runners::search_bar::run_material3_headless_search_bar_suite_goldens_v1();
}

#[test]
fn material3_headless_search_view_suite_goldens_v1() {
    support::headless_golden_runners::search_view::run_material3_headless_search_view_suite_goldens_v1();
}

#[test]
fn material3_headless_carousel_item_suite_goldens_v1() {
    support::headless_golden_runners::carousel_item::run_material3_headless_carousel_item_suite_goldens_v1();
}
