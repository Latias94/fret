pub(super) fn assert_collection_module_routing(collection_source: &str) {
    for needle in [
        "use fret::imui::ImUi;",
        "pub(super) fn render_collection_first_asset_browser_proof(",
        "ui: &mut ImUi<'_, '_, KernelApp>",
        "mod asset_grid;",
        "mod assets;",
        "mod browser_scope;",
        "mod box_select;",
        "mod child_models;",
        "mod chrome;",
        "mod command_buttons;",
        "mod context_menu;",
        "mod derived_state;",
        "mod drag_drop;",
        "mod geometry;",
        "mod import_target;",
        "mod keyboard;",
        "mod lifecycle;",
        "mod models;",
        "mod order_toggle;",
        "mod rename;",
        "mod render_states;",
        "mod runtime_state;",
        "mod selection;",
        "mod status_readouts;",
        "pub(super) use assets::{ProofCollectionAsset, authoring_parity_collection_assets};",
        "pub(super) use chrome::proof_collection_readout_text;",
        "use child_models::{ProofCollectionChildModels, proof_collection_child_models};",
        "use chrome::render_collection_header;",
        "use derived_state::proof_collection_derived_state;",
        "use import_target::render_collection_import_target;",
        "use lifecycle::clear_stale_collection_rename_session;",
        "use order_toggle::render_collection_order_toggle;",
        "use render_states::proof_collection_render_states;",
        "use runtime_state::proof_collection_runtime_state;",
        "render_collection_import_target(ui);",
        "render_collection_order_toggle(",
        "proof_collection_derived_state(",
        "proof_collection_runtime_state(",
        "proof_collection_child_models(&collection_runtime.models)",
        "proof_collection_render_states(",
        "clear_stale_collection_rename_session(",
        "use status_readouts::render_collection_status_readouts;",
        "render_collection_status_readouts(",
        "render_collection_header(ui);",
    ] {
        assert!(
            collection_source.contains(needle),
            "the demo-local collection module should keep the modularized implementation explicit; missing `{needle}`"
        );
    }

    assert!(
        !collection_source.contains("use fret::imui::prelude::*;"),
        "the demo-local collection module should not rely on the broad imui prelude",
    );
}
