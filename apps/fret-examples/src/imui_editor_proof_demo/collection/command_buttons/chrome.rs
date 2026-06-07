use std::sync::Arc;

use fret::imui::kit;

pub(super) fn collection_duplicate_selected_label() -> &'static str {
    "Duplicate selected assets"
}

pub(super) fn collection_rename_active_label() -> &'static str {
    "Rename active asset"
}

pub(super) fn collection_delete_selected_label() -> &'static str {
    "Delete selected assets"
}

pub(super) fn collection_duplicate_selected_button_options(enabled: bool) -> kit::ButtonOptions {
    collection_command_button_options(
        enabled,
        "imui-editor-proof.authoring.imui.collection.duplicate-selected",
    )
}

pub(super) fn collection_rename_active_button_options(enabled: bool) -> kit::ButtonOptions {
    collection_command_button_options(
        enabled,
        "imui-editor-proof.authoring.imui.collection.rename-active",
    )
}

pub(super) fn collection_delete_selected_button_options(enabled: bool) -> kit::ButtonOptions {
    collection_command_button_options(
        enabled,
        "imui-editor-proof.authoring.imui.collection.delete-selected",
    )
}

fn collection_command_button_options(enabled: bool, test_id: &'static str) -> kit::ButtonOptions {
    kit::ButtonOptions {
        enabled,
        test_id: Some(Arc::from(test_id)),
        ..Default::default()
    }
}
