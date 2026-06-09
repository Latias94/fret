pub(super) fn assert_command_buttons_owner_split(
    command_buttons_source: &str,
    command_buttons_actions_source: &str,
    command_buttons_chrome_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionCommandButtonModels",
        "pub(super) struct ProofCollectionCommandButtonState",
        "pub(super) fn render_collection_command_buttons(",
        "mod actions;",
        "mod chrome;",
        "collection_duplicate_selected_label()",
        "collection_duplicate_selected_button_options(!state.selection.is_empty())",
        "collection_rename_active_label()",
        "collection_rename_active_button_options(state.rename_ready_session.is_some())",
        "collection_delete_selected_label()",
        "collection_delete_selected_button_options(!state.selection.is_empty())",
        "proof_collection_command_button_apply_duplicate(",
        "proof_collection_command_button_begin_rename(",
        "proof_collection_command_button_apply_delete(",
    ] {
        assert!(
            command_buttons_source.contains(needle),
            "the demo-local collection command-buttons owner should keep explicit command button routing separate; missing `{needle}`"
        );
    }
    for needle in [
        "proof_collection_duplicate_status(",
        "proof_collection_delete_status(",
        "proof_collection_begin_inline_rename_in_app(",
        "proof_collection_set_command_status(",
        "models_mut().update",
        "kit::ButtonOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
    ] {
        assert!(
            !command_buttons_source.contains(needle),
            "the demo-local collection command-buttons owner should delegate button chrome/test IDs to command_buttons/chrome.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_command_button_apply_duplicate(",
        "pub(super) fn proof_collection_command_button_begin_rename(",
        "pub(super) fn proof_collection_command_button_apply_delete(",
        "proof_collection_duplicate_status(&duplicate.duplicated_assets)",
        "proof_collection_delete_status(&delete.deleted_assets)",
        "proof_collection_begin_inline_rename_in_app(",
        "app.models_mut().update(&models.assets",
        "app.models_mut().update(&models.selection",
        "app.models_mut().update(&models.keyboard",
        "proof_collection_set_command_status(",
    ] {
        assert!(
            command_buttons_actions_source.contains(needle),
            "the demo-local collection command-buttons actions owner should keep button-triggered state writes explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ui.button_with_options(",
        "collection_duplicate_selected_label()",
        "collection_duplicate_selected_button_options(",
        "collection_rename_active_label()",
        "collection_rename_active_button_options(",
        "collection_delete_selected_label()",
        "collection_delete_selected_button_options(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
    ] {
        assert!(
            !command_buttons_actions_source.contains(needle),
            "the demo-local collection command-buttons actions owner should not take button rendering or selection policy; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn collection_duplicate_selected_label() -> &'static str",
        "pub(super) fn collection_rename_active_label() -> &'static str",
        "pub(super) fn collection_delete_selected_label() -> &'static str",
        "pub(super) fn collection_duplicate_selected_button_options(enabled: bool) -> kit::ButtonOptions",
        "pub(super) fn collection_rename_active_button_options(enabled: bool) -> kit::ButtonOptions",
        "pub(super) fn collection_delete_selected_button_options(enabled: bool) -> kit::ButtonOptions",
        "kit::ButtonOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"imui-editor-proof.authoring.imui.collection.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-active\"",
        "\"imui-editor-proof.authoring.imui.collection.delete-selected\"",
    ] {
        assert!(
            command_buttons_chrome_source.contains(needle),
            "the demo-local collection command-buttons chrome owner should keep button label/options/test-id construction explicit; missing `{needle}`"
        );
    }
}
