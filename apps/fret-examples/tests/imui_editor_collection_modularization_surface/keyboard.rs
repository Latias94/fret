pub(super) fn assert_keyboard_owner_split(keyboard_source: &str, keyboard_actions_source: &str) {
    for needle in [
        "pub(super) struct ProofCollectionKeyboardHandlerModels",
        "pub(super) fn install_collection_keyboard_handler(",
        "mod actions;",
        "cx.key_on_key_down_for(",
        "proof_collection_keyboard_selection(",
        "proof_collection_keyboard_apply_delete(",
        "proof_collection_keyboard_begin_rename(",
        "proof_collection_keyboard_apply_select_all(",
        "proof_collection_keyboard_apply_duplicate(",
        "proof_collection_keyboard_apply_navigation(",
    ] {
        assert!(
            keyboard_source.contains(needle),
            "the demo-local collection keyboard owner should keep scope keyboard dispatch explicit; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_delete_status(",
        "proof_collection_duplicate_status(",
        "proof_collection_select_all_status(",
        "proof_collection_rename_ready_status(",
        "host.update_model(&models.assets",
        "host.update_model(&models.selection",
        "host.update_model(&models.keyboard",
        "host.update_model(&models.command_status",
        "host.notify(acx);",
    ] {
        assert!(
            !keyboard_source.contains(needle),
            "the demo-local collection keyboard owner should delegate app-state mutation/status writes to keyboard/actions.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_keyboard_apply_delete(",
        "pub(super) fn proof_collection_keyboard_begin_rename(",
        "pub(super) fn proof_collection_keyboard_apply_select_all(",
        "pub(super) fn proof_collection_keyboard_apply_duplicate(",
        "pub(super) fn proof_collection_keyboard_apply_navigation(",
        "proof_collection_delete_status(&delete.deleted_assets)",
        "proof_collection_duplicate_status(&duplicate.duplicated_assets)",
        "proof_collection_select_all_status(next_selection.selected_count())",
        "proof_collection_rename_ready_status(",
        "host.update_model(&models.assets",
        "host.update_model(&models.selection",
        "host.update_model(&models.keyboard",
        "host.update_model(&models.command_status",
        "host.notify(acx);",
    ] {
        assert!(
            keyboard_actions_source.contains(needle),
            "the demo-local collection keyboard actions owner should keep app-state mutation explicit; missing `{needle}`"
        );
    }
    for needle in [
        "cx.key_on_key_down_for(",
        "proof_collection_delete_key_matches(",
        "proof_collection_rename_shortcut_matches(",
        "proof_collection_select_all_shortcut_matches(",
        "proof_collection_duplicate_shortcut_matches(",
        "proof_collection_keyboard_selection(",
        "proof_collection_assets_in_visible_order(",
        "host.models_mut().read(",
    ] {
        assert!(
            !keyboard_actions_source.contains(needle),
            "the demo-local collection keyboard actions owner should not take key matching, snapshot reads, or selection derivation policy; unexpected `{needle}`"
        );
    }
}
