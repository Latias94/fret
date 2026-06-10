pub(super) fn assert_child_models_owner_split(collection_source: &str, child_models_source: &str) {
    for needle in [
        "pub(super) struct ProofCollectionChildModels",
        "pub(super) fn proof_collection_child_models(",
        "models: &ProofCollectionRuntimeModels",
        "command_buttons: ProofCollectionCommandButtonModels {",
        "browser_scope: ProofCollectionBrowserScopeModels {",
        "context_menu: ProofCollectionContextMenuModels {",
        "assets: models.assets.clone()",
        "selection: models.selection.clone()",
        "keyboard: models.keyboard.clone()",
        "rename_session: models.rename_session.clone()",
        "scroll: models.scroll.clone()",
    ] {
        assert!(
            child_models_source.contains(needle),
            "the demo-local collection child-model owner should keep child model bundle projection explicit; missing `{needle}`"
        );
    }

    for needle in [
        "ProofCollectionCommandButtonModels {",
        "ProofCollectionBrowserScopeModels {",
        "ProofCollectionContextMenuModels {",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route child model bundle projection through collection/child_models.rs; unexpected `{needle}`"
        );
    }
}
