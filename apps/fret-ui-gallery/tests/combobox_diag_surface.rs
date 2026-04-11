#[test]
fn combobox_demo_narrow_diag_script_waits_for_stable_listbox_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-narrow-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-combobox-demo-trigger\"",
        "\"ui-gallery-combobox-demo-input\"",
        "\"ui-gallery-combobox-demo-listbox\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"type\": \"wait_overlay_placement_trace\"",
        "\"ui-gallery-combobox-demo-open-narrow\"",
    ] {
        assert!(
            script.contains(needle),
            "combobox narrow diag script should keep the open-chain and placement evidence stable; missing `{needle}`",
        );
    }
}
