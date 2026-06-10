pub(super) fn assert_import_target_owner_split(
    collection_source: &str,
    import_target_source: &str,
) {
    for needle in [
        "pub(super) fn render_collection_import_target(",
        "authoring_parity_collection_drop_status_model(ui.cx_mut())",
        "ui.button_with_options(",
        "ui.drop_target::<ProofCollectionDragPayload>(import_trigger)",
        "proof_collection_drop_status(\"Delivered\", &payload)",
        "proof_collection_drop_status(\"Preview\", &payload)",
        "\"Compatible collection drag active\"",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            import_target_source.contains(needle),
            "the demo-local collection import-target owner should keep import drop/status UI explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionDragPayload",
        "proof_collection_drop_status(",
        "authoring_parity_collection_drop_status_model",
        "ui.drop_target::<",
        "\"imui-editor-proof.authoring.imui.collection.import-target\"",
        "\"imui-editor-proof.authoring.imui.collection.drop-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route import target/drop-status UI through collection/import_target.rs; unexpected `{needle}`"
        );
    }
}
