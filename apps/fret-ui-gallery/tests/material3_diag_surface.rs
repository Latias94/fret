#![cfg(feature = "gallery-material3")]

const AUTOCOMPLETE_FILTERING_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-filtering.json"
);
const SEARCH_VIEW_SCREENSHOTS_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-screenshots.json"
);
const TEXT_FIELD_HOVER_LABEL_COLOR_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-text-field-hover-label-color-expressive-screenshots.json"
);
const BUTTON_SIZES_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/button/ui-gallery-material3-button-sizes-screenshots.json"
);
const CHECKBOX_TRISTATE_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json"
);
const AUTOCOMPLETE_LEADING_ICON_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/forms/ui-gallery-material3-autocomplete-leading-icon-screenshots.json"
);
const AUTOCOMPLETE_OPTION_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-option-chrome-fill.json"
);
const EXPOSED_DROPDOWN_FILTERING_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-exposed-dropdown-filtering.json"
);
const TEXT_FIELD_ICONS_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/forms/ui-gallery-material3-text-field-icons-screenshots.json"
);
const AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-dialog-nested-overlay.json"
);
const DIALOG_FOCUS_TRAP_RESTORE_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json"
);
const DIALOG_SELECT_NESTED_OVERLAY_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-select-nested-overlay.json"
);
const SELECT_A11Y_PARITY_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json"
);
const SELECT_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-item-chrome-fill.json"
);
const SELECT_RICH_OPTIONS_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-rich-options-screenshots.json"
);
const SELECT_POSITIONING_TRANSFORM_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-positioning-transform-screenshots.json"
);
const SELECT_MENU_WIDTH_FLOOR_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-menu-width-floor-screenshots.json"
);
const SELECT_TYPEAHEAD_DELAY_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-typeahead-delay.json"
);
const LIST_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-list-item-chrome-fill.json"
);
const MENU_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-item-chrome-fill.json"
);
const NAVIGATION_BAR_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-bar-item-chrome-fill.json"
);
const NAVIGATION_BAR_INDICATOR_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-bar-indicator-pixels-changed-fixed-frame-delta.json"
);
const NAVIGATION_DRAWER_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json"
);
const NAVIGATION_RAIL_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-rail-item-chrome-fill.json"
);
const NAVIGATION_RAIL_INDICATOR_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-rail-indicator-pixels-changed-fixed-frame-delta.json"
);
const TABS_ITEM_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-tabs-item-chrome-fill.json"
);
const TABS_INDICATOR_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-tabs-indicator-pixels-changed-fixed-frame-delta.json"
);
const TIME_PICKER_CHROME_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-time-picker-chrome-fill.json"
);

#[test]
fn material3_field_surface_diags_use_direct_start_pages() {
    for (label, script, start_page) in [
        (
            "button sizes",
            BUTTON_SIZES_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_button\"",
        ),
        (
            "checkbox tristate",
            CHECKBOX_TRISTATE_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_checkbox\"",
        ),
        (
            "autocomplete filtering",
            AUTOCOMPLETE_FILTERING_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "autocomplete leading icon",
            AUTOCOMPLETE_LEADING_ICON_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "autocomplete option chrome",
            AUTOCOMPLETE_OPTION_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "exposed dropdown filtering",
            EXPOSED_DROPDOWN_FILTERING_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "search view screenshots",
            SEARCH_VIEW_SCREENSHOTS_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_state_matrix\"",
        ),
        (
            "text field hover label color",
            TEXT_FIELD_HOVER_LABEL_COLOR_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_text_field\"",
        ),
        (
            "text field icons",
            TEXT_FIELD_ICONS_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_text_field\"",
        ),
        (
            "autocomplete dialog nested overlay",
            AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "dialog focus trap restore",
            DIALOG_FOCUS_TRAP_RESTORE_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_dialog\"",
        ),
        (
            "dialog select nested overlay",
            DIALOG_SELECT_NESTED_OVERLAY_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_dialog\"",
        ),
        (
            "select a11y parity",
            SELECT_A11Y_PARITY_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "select item chrome",
            SELECT_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "select rich options",
            SELECT_RICH_OPTIONS_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "select positioning transform",
            SELECT_POSITIONING_TRANSFORM_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "select menu width floor",
            SELECT_MENU_WIDTH_FLOOR_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "select typeahead delay",
            SELECT_TYPEAHEAD_DELAY_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_select\"",
        ),
        (
            "list item chrome",
            LIST_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_list\"",
        ),
        (
            "menu item chrome",
            MENU_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_menu\"",
        ),
        (
            "navigation bar item chrome",
            NAVIGATION_BAR_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_navigation_bar\"",
        ),
        (
            "navigation bar indicator",
            NAVIGATION_BAR_INDICATOR_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_navigation_bar\"",
        ),
        (
            "navigation drawer item chrome",
            NAVIGATION_DRAWER_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_navigation_drawer\"",
        ),
        (
            "navigation rail item chrome",
            NAVIGATION_RAIL_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_navigation_rail\"",
        ),
        (
            "navigation rail indicator",
            NAVIGATION_RAIL_INDICATOR_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_navigation_rail\"",
        ),
        (
            "tabs item chrome",
            TABS_ITEM_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_tabs\"",
        ),
        (
            "tabs indicator",
            TABS_INDICATOR_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_tabs\"",
        ),
        (
            "time picker chrome",
            TIME_PICKER_CHROME_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_time_picker\"",
        ),
    ] {
        assert!(
            script.contains(start_page),
            "{label} diag should start from the dedicated Material3 page"
        );
    }
}

#[test]
fn material3_field_surface_diags_keep_explicit_content_scroll_anchors() {
    for (label, script, scroll_container, target) in [
        (
            "autocomplete filtering",
            AUTOCOMPLETE_FILTERING_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-autocomplete\"",
        ),
        (
            "autocomplete leading icon",
            AUTOCOMPLETE_LEADING_ICON_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-autocomplete\"",
        ),
        (
            "autocomplete option chrome",
            AUTOCOMPLETE_OPTION_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-autocomplete\"",
        ),
        (
            "exposed dropdown filtering",
            EXPOSED_DROPDOWN_FILTERING_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-exposed-dropdown\"",
        ),
        (
            "search view screenshots",
            SEARCH_VIEW_SCREENSHOTS_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-search-view\"",
        ),
        (
            "text field hover label color",
            TEXT_FIELD_HOVER_LABEL_COLOR_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-text-field\"",
        ),
        (
            "text field icons",
            TEXT_FIELD_ICONS_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-text-field-icons\"",
        ),
        (
            "autocomplete dialog nested overlay",
            AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-autocomplete-dialog-open\"",
        ),
        (
            "dialog focus trap restore",
            DIALOG_FOCUS_TRAP_RESTORE_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-dialog-open\"",
        ),
        (
            "dialog select nested overlay",
            DIALOG_SELECT_NESTED_OVERLAY_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-dialog-open\"",
        ),
        (
            "select a11y parity",
            SELECT_A11Y_PARITY_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-select-typeahead\"",
        ),
        (
            "select item chrome",
            SELECT_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-scroll\" }",
            "\"id\": \"ui-gallery-material3-select\"",
        ),
        (
            "select rich options",
            SELECT_RICH_OPTIONS_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-select-rich\"",
        ),
        (
            "select positioning transform",
            SELECT_POSITIONING_TRANSFORM_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-select-transformed\"",
        ),
        (
            "select menu width floor",
            SELECT_MENU_WIDTH_FLOOR_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-select-unclamped\"",
        ),
        (
            "select typeahead delay",
            SELECT_TYPEAHEAD_DELAY_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-select-typeahead-delay-1000\"",
        ),
        (
            "list item chrome",
            LIST_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-standard-list-item-alpha\"",
        ),
        (
            "menu item chrome",
            MENU_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-menu-trigger\"",
        ),
        (
            "navigation bar item chrome",
            NAVIGATION_BAR_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-nav-search\"",
        ),
        (
            "navigation bar indicator",
            NAVIGATION_BAR_INDICATOR_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-nav-settings\"",
        ),
        (
            "navigation drawer item chrome",
            NAVIGATION_DRAWER_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-drawer-search\"",
        ),
        (
            "navigation rail item chrome",
            NAVIGATION_RAIL_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-rail-search\"",
        ),
        (
            "navigation rail indicator",
            NAVIGATION_RAIL_INDICATOR_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-rail-settings\"",
        ),
        (
            "tabs item chrome",
            TABS_ITEM_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-tab-overview\"",
        ),
        (
            "tabs indicator",
            TABS_INDICATOR_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-tab-overview\"",
        ),
        (
            "time picker chrome",
            TIME_PICKER_CHROME_DIAG,
            "\"container\": { \"kind\": \"test_id\", \"id\": \"ui-gallery-content-viewport\" }",
            "\"id\": \"ui-gallery-material3-time-picker-open\"",
        ),
    ] {
        assert!(
            script.contains("\"type\": \"scroll_into_view\""),
            "{label} diag should use explicit scroll_into_view before interacting with content"
        );
        assert!(
            script.contains(scroll_container),
            "{label} diag should scroll the expected gallery content container"
        );
        assert!(
            script.contains(target),
            "{label} diag should keep the stable target anchor explicit"
        );
    }
}

#[test]
fn material3_control_surface_diags_drop_nav_search_hops_when_start_page_is_owned() {
    for (label, script) in [
        ("button sizes", BUTTON_SIZES_DIAG),
        ("checkbox tristate", CHECKBOX_TRISTATE_DIAG),
        ("autocomplete leading icon", AUTOCOMPLETE_LEADING_ICON_DIAG),
        (
            "autocomplete option chrome",
            AUTOCOMPLETE_OPTION_CHROME_DIAG,
        ),
        (
            "exposed dropdown filtering",
            EXPOSED_DROPDOWN_FILTERING_DIAG,
        ),
        ("text field icons", TEXT_FIELD_ICONS_DIAG),
        (
            "autocomplete dialog nested overlay",
            AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG,
        ),
        ("dialog focus trap restore", DIALOG_FOCUS_TRAP_RESTORE_DIAG),
        (
            "dialog select nested overlay",
            DIALOG_SELECT_NESTED_OVERLAY_DIAG,
        ),
        (
            "text field hover label color",
            TEXT_FIELD_HOVER_LABEL_COLOR_DIAG,
        ),
        ("select a11y parity", SELECT_A11Y_PARITY_DIAG),
        ("select item chrome", SELECT_ITEM_CHROME_DIAG),
        ("select rich options", SELECT_RICH_OPTIONS_DIAG),
        (
            "select positioning transform",
            SELECT_POSITIONING_TRANSFORM_DIAG,
        ),
        ("select menu width floor", SELECT_MENU_WIDTH_FLOOR_DIAG),
        ("select typeahead delay", SELECT_TYPEAHEAD_DELAY_DIAG),
        ("list item chrome", LIST_ITEM_CHROME_DIAG),
        ("menu item chrome", MENU_ITEM_CHROME_DIAG),
        (
            "navigation bar item chrome",
            NAVIGATION_BAR_ITEM_CHROME_DIAG,
        ),
        ("navigation bar indicator", NAVIGATION_BAR_INDICATOR_DIAG),
        (
            "navigation drawer item chrome",
            NAVIGATION_DRAWER_ITEM_CHROME_DIAG,
        ),
        (
            "navigation rail item chrome",
            NAVIGATION_RAIL_ITEM_CHROME_DIAG,
        ),
        ("navigation rail indicator", NAVIGATION_RAIL_INDICATOR_DIAG),
        ("tabs item chrome", TABS_ITEM_CHROME_DIAG),
        ("tabs indicator", TABS_INDICATOR_DIAG),
        ("time picker chrome", TIME_PICKER_CHROME_DIAG),
    ] {
        assert!(
            !script.contains("\"id\": \"ui-gallery-nav-search\""),
            "{label} diag should not route through nav search once it owns a start page"
        );
    }
}
