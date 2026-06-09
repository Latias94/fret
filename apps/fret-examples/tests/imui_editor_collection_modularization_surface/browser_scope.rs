pub(super) fn assert_browser_scope_owner_split(
    browser_scope_source: &str,
    browser_scope_chrome_source: &str,
    browser_scope_asset_grid_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeModels",
        "pub(super) struct ProofCollectionBrowserScopeState",
        "pub(super) fn render_collection_browser_scope(",
        "ui.child_region_with_options(",
        "mod asset_grid;",
        "mod chrome;",
        "render_collection_browser_scope_asset_grid(",
        "collection_browser_child_region_options(",
        "collection_browser_box_select_marquee(",
        "collection_browser_box_select_scope_id()",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
    ] {
        assert!(
            browser_scope_source.contains(needle),
            "the demo-local collection browser-scope owner should keep child-region pointer runtime explicit; missing `{needle}`"
        );
    }
    for needle in [
        "kit::ChildRegionOptions",
        "kit::ScrollOptions",
        "fret_ui_kit::LayoutRefinement::default()",
        "pub(super) fn collection_browser_child_region_id() -> &'static str",
        "pub(super) fn collection_browser_child_region_options(",
        "pub(super) fn collection_browser_box_select_scope_id() -> &'static str",
        "pub(super) fn collection_browser_box_select_marquee(",
        "\"imui-editor-proof.authoring.imui.collection.browser\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.viewport\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.content\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
        "Color { a: 0.14, ..ring }",
        "Color { a: 0.88, ..ring }",
        ".absolute()",
        ".border_1()",
    ] {
        assert!(
            browser_scope_chrome_source.contains(needle),
            "the demo-local collection browser-scope chrome owner should keep child-region option/test-id and marquee chrome construction explicit; missing `{needle}`"
        );
    }
    for needle in [
        "kit::ChildRegionOptions",
        "kit::ScrollOptions",
        "\"imui-editor-proof.authoring.imui.collection.browser\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.viewport\"",
        "\"imui-editor-proof.authoring.imui.collection.browser.content\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.scope\"",
        "\"imui-editor-proof.authoring.imui.collection.box-select.marquee\"",
        "Color { a:",
        ".border_1()",
        "render_collection_asset_grid(",
        "ProofCollectionAssetGridModels {",
        "ProofCollectionAssetGridState {",
    ] {
        assert!(
            !browser_scope_source.contains(needle),
            "the demo-local collection browser-scope owner should delegate chrome construction to browser_scope/chrome.rs and asset-grid mounting to browser_scope/asset_grid.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) struct ProofCollectionBrowserScopeAssetGridModels",
        "pub(super) struct ProofCollectionBrowserScopeAssetGridState",
        "pub(super) fn render_collection_browser_scope_asset_grid(",
        "fret_ui_kit::ui::container_build(",
        "imui_build(cx, out, |ui| {",
        "render_collection_asset_grid(",
        "ProofCollectionAssetGridModels {",
        "ProofCollectionAssetGridState {",
        ".w_full()",
        ".into_element(cx)",
    ] {
        assert!(
            browser_scope_asset_grid_source.contains(needle),
            "the demo-local collection browser-scope asset-grid owner should keep grid element mounting explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ui.child_region_with_options(",
        "proof_collection_browser_scope_pointer_props()",
        "install_collection_browser_scope_input_runtime(",
        "collection_browser_box_select_marquee(",
        "proof_collection_box_select_active_rect(",
        "kit::ChildRegionOptions",
        "cx.pointer_region(",
    ] {
        assert!(
            !browser_scope_asset_grid_source.contains(needle),
            "the demo-local collection browser-scope asset-grid owner should not take child-region, pointer runtime, or marquee chrome responsibilities; unexpected `{needle}`"
        );
    }
}
