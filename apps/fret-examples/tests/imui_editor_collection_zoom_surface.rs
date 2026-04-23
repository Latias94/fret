#[test]
fn imui_editor_proof_demo_keeps_collection_zoom_app_owned_and_explicit() {
    let source = include_str!("../src/imui_editor_proof_demo/collection.rs");

    for needle in [
        "struct ProofCollectionLayoutMetrics {",
        "struct ProofCollectionZoomUpdate {",
        "fn proof_collection_layout_metrics(",
        "fn proof_collection_zoom_line(",
        "fn proof_collection_zoom_request(",
        "fn authoring_parity_collection_zoom_model<H: UiHost>(",
        "fn authoring_parity_collection_scroll_handle<H: UiHost>(",
        "imui_editor_proof_demo.model.authoring_parity.collection_zoom",
        "imui_editor_proof_demo.state.authoring_parity.collection_scroll_handle",
        "\"Primary+Wheel zoom stays app-owned: {} px target tiles across {} column(s), with hovered rows staying anchored inside the collection proof.\"",
        "collection_scroll_handle.viewport_size().width",
        "handle: Some(collection_scroll_handle.clone())",
        "proof_collection_zoom_request(",
        "collection_layout.columns",
        "collection_scroll_handle_for_wheel.set_offset(update.next_scroll_offset);",
    ] {
        assert!(
            source.contains(needle),
            "imui_editor_proof_demo should keep the collection zoom proof explicit and app-owned; missing `{needle}`"
        );
    }

    for needle in [
        "fret_ui_kit::imui::collection_zoom",
        "pub fn collection_zoom",
        "pub fn collection_layout_metrics",
        "struct ImUiCollectionZoom",
    ] {
        assert!(
            !source.contains(needle),
            "imui_editor_proof_demo should not pretend the collection zoom slice is already a shared helper; unexpected `{needle}`"
        );
    }
}
