pub(super) fn assert_derived_state_owner_split(
    collection_source: &str,
    derived_state_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionDerivedState",
        "pub(super) fn proof_collection_derived_state(",
        "stored_assets: &[ProofCollectionAsset]",
        "reverse_order: bool",
        "proof_collection_assets_in_visible_order(",
        "Arc::<[ProofCollectionAsset]>::from(stored_assets.to_vec())",
        "let keys = assets",
        ".map(|asset| asset.id.clone())",
        ".collect::<Vec<_>>();",
        "proof_collection_active_id(&keys, selection, keyboard)",
        "proof_collection_begin_rename_session(&assets, selection, keyboard)",
        "rename_ready_session",
    ] {
        assert!(
            derived_state_source.contains(needle),
            "the demo-local collection derived-state owner should keep visible asset/key/active/rename-ready projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_assets_in_visible_order(",
        "proof_collection_active_id(",
        "proof_collection_begin_rename_session(",
        "let collection_keys =",
        "let collection_active_id =",
        "let collection_rename_ready_session =",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route derived visible state through collection/derived_state.rs; unexpected `{needle}`"
        );
    }
}
