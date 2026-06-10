pub(super) fn assert_lifecycle_owner_split(collection_source: &str, lifecycle_source: &str) {
    for needle in [
        "pub(super) fn clear_stale_collection_rename_session(",
        "models: &ProofCollectionRuntimeModels",
        "snapshot: &ProofCollectionRuntimeSnapshot",
        "assets: &[ProofCollectionAsset]",
        "snapshot.rename_session.as_ref()",
        "!assets.iter().any(|asset| asset.id == session.target_id)",
        ".update(&models.rename_session, |state| *state = None)",
        ".update(&models.rename_focus_pending, |state| *state = false)",
    ] {
        assert!(
            lifecycle_source.contains(needle),
            "the demo-local collection lifecycle owner should keep stale rename cleanup explicit; missing `{needle}`"
        );
    }

    for needle in [
        "snapshot.rename_session.as_ref()",
        "models.rename_session",
        "models.rename_focus_pending",
        ".update(&collection_runtime.models.rename_session",
        ".update(&collection_runtime.models.rename_focus_pending",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route stale rename cleanup through collection/lifecycle.rs; unexpected `{needle}`"
        );
    }
}
