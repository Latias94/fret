fn compact(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

pub(super) fn assert_model_owner_boundary(
    collection_source: &str,
    model_owner_source: &str,
    command_buttons_actions_source: &str,
    context_menu_source: &str,
    context_menu_actions_source: &str,
    asset_grid_actions_source: &str,
    rename_source: &str,
) {
    for needle in [
        "mod model_owner;",
        "mod command_buttons;",
        "mod context_menu;",
        "mod asset_grid;",
        "mod rename;",
    ] {
        assert!(
            collection_source.contains(needle),
            "the collection module should expose a local model-owner boundary next to the action modules; missing `{needle}`"
        );
    }

    for needle in [
        "pub(super) struct ProofCollectionModelOwner<'a>",
        "models: &'a mut ModelStore",
        "fn update<T: Any, R>(",
        "pub(super) fn apply_duplicate(",
        "pub(super) fn apply_delete(",
        "pub(super) fn begin_inline_rename(",
        "pub(super) fn publish_active_focus_target(",
        "pub(super) fn activate_asset(",
        "pub(super) fn apply_context_menu(",
        "pub(super) fn clear_context_menu_anchor(",
        "proof_collection_duplicate_status(",
        "proof_collection_delete_status(",
        "proof_collection_rename_ready_status(",
    ] {
        assert!(
            model_owner_source.contains(needle),
            "the collection model owner should own shared-model mutation semantics; missing `{needle}`"
        );
    }

    for source in [
        command_buttons_actions_source,
        context_menu_source,
        context_menu_actions_source,
        asset_grid_actions_source,
        rename_source,
    ] {
        let compact_source = compact(source);
        for forbidden in [
            "app.models_mut().update(",
            "app.models_mut().update::<",
            "app.models_mut().update_any(",
            "app.models_mut().update_any::<",
            ".models_mut().update(",
            ".models_mut().update::<",
            ".models_mut().update_any(",
            ".models_mut().update_any::<",
            "ModelStore::update(",
            "ModelStore::update::<",
            "ModelStore::update_any(",
            "ModelStore::update_any::<",
        ] {
            assert!(
                !compact_source.contains(forbidden),
                "collection action/render hub sources should route shared-model writes through ProofCollectionModelOwner; unexpected `{forbidden}`"
            );
        }
    }
}
