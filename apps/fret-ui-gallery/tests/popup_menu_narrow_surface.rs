#[test]
fn context_menu_demo_snippet_uses_a_unique_overlay_panel_test_id() {
    let demo = include_str!("../src/ui/snippets/context_menu/demo.rs");

    assert!(
        demo.contains("\"ui-gallery-context-menu-demo-panel\""),
        "context-menu demo snippet should expose a unique overlay panel test id for diag scripts",
    );
    assert!(
        !demo.contains("\"ui-gallery-context-menu-demo-content\""),
        "context-menu demo snippet should not reuse the DocSection content test id for the open menu panel",
    );
}

#[test]
fn popup_menu_narrow_sweep_covers_select_combobox_context_menu_and_dropdown_menu() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json"
    );

    for needle in [
        "\"ui-gallery-select-shadcn-demo-trigger\"",
        "\"select-scroll-viewport\"",
        "\"ui-gallery-combobox-demo-trigger\"",
        "\"ui-gallery-combobox-demo-input\"",
        "\"ui-gallery-combobox-demo-listbox\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_overlay_placement_trace\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"ui-gallery-context-menu-demo-trigger\"",
        "\"ui-gallery-context-menu-demo-panel\"",
        "\"ui-gallery-dropdown-menu-demo-trigger.chrome\"",
        "\"ui-gallery-dropdown-menu-demo-profile.chrome\"",
        "\"ui-gallery-popup-menu-narrow-sweep.dropdown-menu\"",
        "\"bounds_within_window\"",
    ] {
        assert!(
            script.contains(needle),
            "popup/menu narrow sweep should keep the selector stable; missing `{needle}`",
        );
    }
}

#[test]
fn select_and_combobox_demo_snippets_clamp_width_inside_narrow_doc_columns() {
    let select_demo = include_str!("../src/ui/snippets/select/demo.rs");
    let combobox_demo = include_str!("../src/ui/snippets/combobox/conformance_demo.rs");
    let normalized_select_demo = normalize_ws(select_demo);
    let normalized_combobox_demo = normalize_ws(combobox_demo);

    assert!(
        normalized_select_demo.contains(".w_full().max_w(Px(180.0)).min_w_0()"),
        "select demo snippet should clamp to the available doc-column width while keeping the upstream 180px max width",
    );
    assert!(
        !select_demo.contains(".w_px(Px(180.0))"),
        "select demo snippet should not force a fixed-width trigger that can overflow the narrow docs column",
    );

    assert!(
        normalized_combobox_demo.contains(".w_full().max_w(Px(260.0)).min_w_0()"),
        "combobox conformance demo should clamp to the available doc-column width while keeping the upstream 260px max width",
    );
    assert!(
        !combobox_demo.contains(".width_px(Px(260.0))"),
        "combobox conformance demo should not force a fixed-width trigger that can overflow the narrow docs column",
    );
}
fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}
