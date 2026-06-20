#[test]
fn combobox_long_list_focused_perf_script_starts_on_focused_section_without_scroll_probe() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/perf/ui-gallery-combobox-long-list-focused-filter-select-steady.json"
    );

    for needle in [
        "\"name\": \"ui-gallery-combobox-long-list-focused-filter-select-steady\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"combobox\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"docsec-long-list-content\"",
        "\"ui-gallery-combobox-long-list-trigger\"",
        "\"ui-gallery-combobox-long-list-input\"",
        "\"ui-gallery-combobox-long-list-listbox\"",
        "\"ui-gallery-combobox-long-list-item-249\"",
        "\"ui-gallery-combobox-long-list-focused-filter-select-steady\"",
    ] {
        assert!(
            script.contains(needle),
            "focused combobox long-list perf script should keep the focused section open/filter/select path stable; missing `{needle}`"
        );
    }

    assert!(
        !script.contains("\"type\": \"scroll_into_view\""),
        "focused combobox long-list perf script should not depend on a whole-page scroll probe before the long-list trigger appears"
    );
}
