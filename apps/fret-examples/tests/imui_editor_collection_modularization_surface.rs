#[test]
fn imui_editor_proof_demo_routes_collection_proof_through_demo_local_module() {
    let demo_source = include_str!("../src/imui_editor_proof_demo.rs");
    let collection_source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "mod collection;",
        "collection::render_collection_first_asset_browser_proof(ui);",
        "collection::authoring_parity_collection_assets()",
    ] {
        assert!(
            demo_source.contains(needle),
            "imui_editor_proof_demo should keep the collection proof routed through the demo-local module; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_assets_in_visible_order(",
        "fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "struct ProofCollectionAsset {",
        "fn proof_collection_drag_rect_normalizes_drag_direction()",
    ] {
        assert!(
            !demo_source.contains(needle),
            "imui_editor_proof_demo should not keep the collection implementation inline after modularization; unexpected `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn authoring_parity_collection_assets() -> Arc<[ProofCollectionAsset]> {",
        "pub(super) fn render_collection_first_asset_browser_proof(",
        "ui: &mut fret_imui::ImUi<'_, '_, KernelApp>,",
        "#[cfg(test)]",
        "fn proof_collection_drag_rect_normalizes_drag_direction() {",
    ] {
        assert!(
            collection_source.contains(needle),
            "the demo-local collection module should keep the modularized implementation and test floor explicit; missing `{needle}`"
        );
    }
}
