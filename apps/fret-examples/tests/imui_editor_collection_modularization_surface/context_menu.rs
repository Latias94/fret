fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

pub(super) fn assert_context_menu_owner_split(
    context_menu_source: &str,
    context_menu_actions_source: &str,
    context_menu_chrome_source: &str,
) {
    let compact_context_source = compact(context_menu_source);
    let compact_actions_source = compact(context_menu_actions_source);

    for needle in [
        "pub(super) struct ProofCollectionContextMenuModels",
        "pub(super) fn render_collection_context_menu(",
        "mod actions;",
        "mod chrome;",
        "proof_collection_context_menu_apply_duplicate(",
        "proof_collection_context_menu_begin_rename(",
        "proof_collection_context_menu_apply_delete(",
        "collection_context_menu_popup_id()",
        "collection_context_menu_selection_readout_id()",
        "collection_context_menu_duplicate_selected_options(",
        "collection_context_menu_rename_active_options(",
        "collection_context_menu_delete_selected_options(",
        "collection_context_menu_dismiss_options(",
        "ui.begin_popup_menu(",
    ] {
        assert!(
            context_menu_source.contains(needle),
            "the demo-local collection context-menu owner should keep popup workflow explicit; missing `{needle}`"
        );
    }
    for needle in [
        "ProofCollectionModelOwner::new(ui.cx_mut().app.models_mut()).clear_context_menu_anchor(&models.anchor)",
    ] {
        assert!(
            compact_context_source.contains(&compact(needle)),
            "the demo-local collection context-menu owner should keep popup workflow explicit; missing `{needle}`"
        );
    }
    for needle in [
        "PROOF_COLLECTION_CONTEXT_MENU_POPUP_ID",
        "kit::MenuItemOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"Dismiss quick actions\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.rename\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.delete-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.dismiss\"",
        "proof_collection_duplicate_status(",
        "proof_collection_delete_status(",
        "proof_collection_begin_inline_rename_in_app(",
        "app.models_mut().update(&models.assets",
        "app.models_mut().update(&models.selection",
        "app.models_mut().update(&models.keyboard",
        "app.models_mut().update(&models.command_status",
        ".update(&models.anchor",
    ] {
        assert!(
            !context_menu_source.contains(needle),
            "the demo-local collection context-menu owner should delegate menu chrome/test IDs to context_menu/chrome.rs; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn proof_collection_context_menu_apply_duplicate(",
        "pub(super) fn proof_collection_context_menu_begin_rename(",
        "pub(super) fn proof_collection_context_menu_apply_delete(",
        "use super::super::model_owner::ProofCollectionModelOwner;",
    ] {
        assert!(
            context_menu_actions_source.contains(needle),
            "the demo-local collection context-menu actions owner should route app-state mutation through the collection model owner; missing `{needle}`"
        );
    }
    for needle in [
        "ProofCollectionModelOwner::new(app.models_mut()).apply_duplicate(",
        "ProofCollectionModelOwner::new(app.models_mut()).begin_inline_rename(",
        "ProofCollectionModelOwner::new(app.models_mut()).apply_delete(",
    ] {
        assert!(
            compact_actions_source.contains(&compact(needle)),
            "the demo-local collection context-menu actions owner should route app-state mutation through the collection model owner; missing `{needle}`"
        );
    }
    for needle in [
        "ui.open_popup_at(",
        "ui.begin_popup_menu(",
        "ui.menu_item_with_options(",
        "collection_context_menu_duplicate_selected_options(",
        "collection_context_menu_rename_active_options(",
        "collection_context_menu_delete_selected_options(",
        "collection_context_menu_dismiss_options(",
        "proof_collection_duplicate_selection(",
        "proof_collection_delete_selection(",
        "proof_collection_begin_rename_session(",
        "kit::MenuItemOptions",
        "proof_collection_duplicate_status(",
        "proof_collection_delete_status(",
        "proof_collection_begin_inline_rename_in_app(",
        "models_mut().update",
    ] {
        assert!(
            !context_menu_actions_source.contains(needle),
            "the demo-local collection context-menu actions owner should not take popup layout, menu chrome, selection derivation policy, or raw model mutation; unexpected `{needle}`"
        );
    }
    for needle in [
        "pub(super) fn collection_context_menu_popup_id() -> &'static str",
        "pub(super) fn collection_context_menu_selection_readout_id() -> &'static str",
        "pub(super) fn collection_context_menu_duplicate_selected_label() -> &'static str",
        "pub(super) fn collection_context_menu_rename_active_label() -> &'static str",
        "pub(super) fn collection_context_menu_delete_selected_label() -> &'static str",
        "pub(super) fn collection_context_menu_dismiss_label() -> &'static str",
        "pub(super) fn collection_context_menu_duplicate_selected_options(",
        "pub(super) fn collection_context_menu_rename_active_options(",
        "pub(super) fn collection_context_menu_delete_selected_options(",
        "pub(super) fn collection_context_menu_dismiss_options(",
        "fn collection_context_menu_action_options(",
        "kit::MenuItemOptions",
        "\"Duplicate selected assets\"",
        "\"Rename active asset\"",
        "\"Delete selected assets\"",
        "\"Dismiss quick actions\"",
        "\"Primary+D\"",
        "\"F2\"",
        "\"Del\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.rename\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.delete-selected\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu.dismiss\"",
    ] {
        assert!(
            context_menu_chrome_source.contains(needle),
            "the demo-local collection context-menu chrome owner should keep popup/menu option/test-id construction explicit; missing `{needle}`"
        );
    }
}
