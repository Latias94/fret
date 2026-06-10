pub(super) fn assert_status_readouts_owner_split(
    collection_source: &str,
    readouts_source: &str,
    readout_status_source: &str,
    status_readouts_source: &str,
) {
    for needle in [
        "pub(super) struct ProofCollectionStatusReadoutState",
        "pub(super) fn render_collection_status_readouts(",
        "proof_collection_assets_line(state.assets)",
        "proof_collection_visible_order_line(state.assets)",
        "proof_collection_selection_line(state.assets, state.selection)",
        "proof_collection_active_line(state.assets, state.selection, state.keyboard)",
        "proof_collection_zoom_line(state.layout)",
        "proof_collection_select_all_line()",
        "proof_collection_rename_line()",
        "proof_collection_context_menu_line()",
        "proof_collection_command_package_line()",
        "proof_collection_rename_status_line(state.rename_status)",
        "proof_collection_command_status_line(state.command_status)",
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            status_readouts_source.contains(needle),
            "the demo-local collection status-readouts owner should keep readout mounting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "mod status;",
        "pub(super) use status::{",
        "proof_collection_command_status_line",
        "proof_collection_delete_status",
        "proof_collection_duplicate_status",
        "proof_collection_rename_cancel_status",
        "proof_collection_rename_commit_status",
        "proof_collection_rename_invalid_status",
        "proof_collection_rename_ready_status",
        "proof_collection_rename_status_line",
        "proof_collection_select_all_status",
        "pub(super) fn proof_collection_selection_line(",
        "pub(super) fn proof_collection_visible_order_line(",
        "pub(super) fn proof_collection_active_line(",
        "pub(super) fn proof_collection_assets_line(",
        "pub(super) fn proof_collection_command_package_line() -> String",
        "pub(super) fn proof_collection_select_all_line() -> String",
        "pub(super) fn proof_collection_rename_line() -> String",
        "pub(super) fn proof_collection_context_menu_line() -> String",
        "proof_collection_selected_assets",
        "proof_collection_active_id",
    ] {
        assert!(
            readouts_source.contains(needle),
            "the demo-local collection readouts hub should keep line readouts and status re-exports explicit; missing `{needle}`"
        );
    }

    for needle in [
        "fn proof_collection_command_status_line(",
        "fn proof_collection_select_all_status(",
        "fn proof_collection_rename_ready_status(",
        "fn proof_collection_rename_commit_status(",
        "fn proof_collection_rename_invalid_status(",
        "fn proof_collection_rename_cancel_status(",
        "fn proof_collection_rename_status_line(",
        "fn proof_collection_duplicate_status(",
        "fn proof_collection_delete_status(",
    ] {
        assert!(
            !readouts_source.contains(needle),
            "the demo-local collection readouts hub should route status formatting through readouts/status.rs; unexpected `{needle}`"
        );
    }

    for needle in [
        "use super::super::ProofCollectionAsset;",
        "pub(in super::super) fn proof_collection_command_status_line(",
        "pub(in super::super) fn proof_collection_select_all_status(",
        "pub(in super::super) fn proof_collection_rename_ready_status(",
        "pub(in super::super) fn proof_collection_rename_commit_status(",
        "pub(in super::super) fn proof_collection_rename_invalid_status(",
        "pub(in super::super) fn proof_collection_rename_cancel_status(",
        "pub(in super::super) fn proof_collection_rename_status_line(",
        "pub(in super::super) fn proof_collection_duplicate_status(",
        "pub(in super::super) fn proof_collection_delete_status(",
        "format!(\"Command status: {status}\")",
        "format!(\"Rename status: {status}\")",
        "Duplicated {} asset(s): {labels}",
        "Deleted {} asset(s): {labels}",
    ] {
        assert!(
            readout_status_source.contains(needle),
            "the demo-local collection readout status owner should keep command/rename/delete/duplicate status formatting explicit; missing `{needle}`"
        );
    }

    for needle in [
        "proof_collection_selection_line(",
        "proof_collection_visible_order_line(",
        "proof_collection_active_line(",
        "proof_collection_command_package_line(",
        "ImUiMultiSelectState",
        "ProofCollectionKeyboardState",
        "proof_collection_selected_assets",
        "proof_collection_active_id",
        "TextField",
        "kit::ButtonOptions",
    ] {
        assert!(
            !readout_status_source.contains(needle),
            "the demo-local collection readout status owner should not take line/projection/UI responsibilities; unexpected `{needle}`"
        );
    }

    for needle in [
        "proof_collection_assets_line(",
        "proof_collection_visible_order_line(",
        "proof_collection_selection_line(",
        "proof_collection_active_line(",
        "proof_collection_zoom_line(",
        "proof_collection_select_all_line(",
        "proof_collection_rename_line(",
        "proof_collection_context_menu_line(",
        "proof_collection_command_package_line(",
        "proof_collection_rename_status_line(",
        "proof_collection_command_status_line(",
        "\"imui-editor-proof.authoring.imui.collection.assets-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.visible-order-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.selection-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.active-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.zoom-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.select-all-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.context-menu-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-package-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.rename-status-readout\"",
        "\"imui-editor-proof.authoring.imui.collection.command-status-readout\"",
    ] {
        assert!(
            !collection_source.contains(needle),
            "the collection root should route status readouts through collection/status_readouts.rs; unexpected `{needle}`"
        );
    }
}
