pub(super) fn assert_selection_duplicate_naming_owner_split(
    selection_duplicate_naming_source: &str,
    selection_duplicate_naming_tests_source: &str,
    selection_duplicate_naming_tests_fixtures_source: &str,
) {
    for needle in [
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "pub(super) struct ProofCollectionDuplicateNameRegistry",
        "pub(super) fn from_assets(stored_assets: &[ProofCollectionAsset]) -> Self",
        "pub(super) fn duplicate_id(&mut self, id: &str) -> Arc<str>",
        "pub(super) fn duplicate_label(&mut self, label: &str) -> Arc<str>",
        "pub(super) fn duplicate_path(&mut self, path: &str) -> Arc<str>",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_duplicate_naming_source.contains(needle),
            "the demo-local collection duplicate naming owner should keep copy-suffix generation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod fixtures;",
        "use fixtures::asset;",
        "ProofCollectionDuplicateNameRegistry::from_assets(&stored_assets)",
        "proof_collection_duplicate_name_registry_uses_unique_copy_suffixes",
    ] {
        assert!(
            selection_duplicate_naming_tests_source.contains(needle),
            "the demo-local collection duplicate naming tests owner should keep copy-suffix registry coverage explicit; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) fn asset(id: &str, label: &str, path: &str) -> ProofCollectionAsset",
        "ProofCollectionAsset {",
        "kind: Arc::from(\"Texture\")",
        "size_kib: 256",
    ] {
        assert!(
            selection_duplicate_naming_tests_fixtures_source.contains(needle),
            "the demo-local collection duplicate naming tests fixture owner should keep asset construction explicit; missing `{needle}`"
        );
    }

    for needle in ["fn asset(id: &str, label: &str, path: &str) -> ProofCollectionAsset"] {
        assert!(
            !selection_duplicate_naming_tests_source.contains(needle),
            "the demo-local collection duplicate naming tests owner should import fixtures instead of defining them; unexpected `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionDuplicateNameRegistry::from_assets(&stored_assets)",
        "proof_collection_duplicate_name_registry_uses_unique_copy_suffixes",
    ] {
        assert!(
            !selection_duplicate_naming_source.contains(needle),
            "the demo-local collection duplicate naming owner should not take naming tests; unexpected `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionDuplicateNameRegistry::from_assets(&stored_assets)",
        "proof_collection_duplicate_name_registry_uses_unique_copy_suffixes",
        "fn proof_collection_duplicate_selection(",
        "fn proof_collection_duplicate_shortcut_matches(",
        "ImUiMultiSelectState",
        "ProofCollectionKeyboardState",
    ] {
        assert!(
            !selection_duplicate_naming_tests_fixtures_source.contains(needle),
            "the demo-local collection duplicate naming tests fixture owner should not take registry behavior or duplicate command flow; unexpected `{needle}`"
        );
    }

    for needle in [
        "struct ProofCollectionDuplicateResult",
        "fn proof_collection_duplicate_selection(",
        "fn proof_collection_duplicate_shortcut_matches(",
        "ImUiMultiSelectState",
        "ProofCollectionKeyboardState",
        "proof_collection_assets_in_visible_order",
    ] {
        assert!(
            !selection_duplicate_naming_source.contains(needle),
            "the demo-local collection duplicate naming owner should not take duplicate command flow; unexpected `{needle}`"
        );
    }
}
