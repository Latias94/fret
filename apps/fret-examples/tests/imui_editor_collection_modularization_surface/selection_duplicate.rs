pub(super) fn assert_selection_duplicate_owner_split(
    selection_duplicate_commands_source: &str,
    selection_duplicate_commands_tests_source: &str,
) {
    for needle in [
        "mod naming;",
        "mod selection;",
        "use naming::ProofCollectionDuplicateNameRegistry;",
        "use selection::proof_collection_duplicate_selection_result;",
        "pub(in super::super::super) struct ProofCollectionDuplicateResult",
        "pub(in super::super::super) fn proof_collection_duplicate_selection(",
        "pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(",
        "proof_collection_duplicate_selection_result(",
        "#[cfg(test)]",
        "mod tests;",
    ] {
        assert!(
            selection_duplicate_commands_source.contains(needle),
            "the demo-local collection duplicate command owner should keep the shortcut/facade and child-owner delegation explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_duplicate_shortcut_matches(",
        "proof_collection_duplicate_shortcut_matches_primary_d_only",
    ] {
        assert!(
            selection_duplicate_commands_tests_source.contains(needle),
            "the demo-local collection duplicate command tests owner should keep shortcut coverage explicit; missing `{needle}`"
        );
    }

    for needle in ["proof_collection_duplicate_shortcut_matches_primary_d_only"] {
        assert!(
            !selection_duplicate_commands_source.contains(needle),
            "the demo-local collection duplicate command owner should not take shortcut tests; unexpected `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_unique_copy_text(",
        "fn proof_collection_duplicate_label_candidate(",
        "fn proof_collection_duplicate_id_candidate(",
        "fn proof_collection_duplicate_path_candidate(",
        "HashSet",
        "HashMap",
        "ProofCollectionDuplicateNameRegistry::from_assets(stored_assets)",
        "name_registry.duplicate_id(asset.id.as_ref())",
        "name_registry.duplicate_label(asset.label.as_ref())",
        "name_registry.duplicate_path(asset.path.as_ref())",
        "proof_collection_active_id(",
        "proof_collection_assets_in_visible_order(",
        "selection_state(",
        "proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy",
    ] {
        assert!(
            !selection_duplicate_commands_source.contains(needle),
            "the demo-local collection duplicate command owner should route naming and selection repair through duplicate child owners; unexpected `{needle}`"
        );
    }
}
