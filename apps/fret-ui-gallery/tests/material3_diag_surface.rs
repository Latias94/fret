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
const AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-dialog-nested-overlay.json"
);
const DIALOG_FOCUS_TRAP_RESTORE_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json"
);

#[test]
fn material3_field_surface_diags_use_direct_start_pages() {
    for (label, script, start_page) in [
        (
            "autocomplete filtering",
            AUTOCOMPLETE_FILTERING_DIAG,
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
            "autocomplete dialog nested overlay",
            AUTOCOMPLETE_DIALOG_NESTED_OVERLAY_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_autocomplete\"",
        ),
        (
            "dialog focus trap restore",
            DIALOG_FOCUS_TRAP_RESTORE_DIAG,
            "\"FRET_UI_GALLERY_START_PAGE\": \"material3_dialog\"",
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
