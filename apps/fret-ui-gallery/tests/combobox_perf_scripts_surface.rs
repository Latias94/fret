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

#[test]
fn combobox_long_list_snippet_keeps_static_item_text_pool() {
    let source = include_str!("../src/ui/snippets/combobox/long_list.rs");

    for needle in [
        "use std::sync::{Arc, OnceLock};",
        "static ITEMS: OnceLock<Vec<LongListItemSpec>> = OnceLock::new();",
        "fn long_list_item_specs() -> &'static [LongListItemSpec]",
        "fn long_list_items() -> Vec<shadcn::ComboboxItem>",
        "let items = long_list_items();",
    ] {
        assert!(
            source.contains(needle),
            "combobox long-list snippet should cache stable item text outside the render hot path; missing `{needle}`",
        );
    }

    let render_body = source
        .split("pub fn render")
        .nth(1)
        .expect("long-list snippet render function");
    assert!(
        !render_body.contains("(0..250)"),
        "combobox long-list render should not rebuild the synthetic text pool every frame",
    );
}
