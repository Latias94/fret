use std::sync::Arc;

use fret::imui::kit;
use fret_runtime::Model;

pub(super) fn collection_context_menu_popup_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.context-menu"
}

pub(super) fn collection_context_menu_selection_readout_id() -> &'static str {
    "imui-editor-proof.authoring.imui.collection.context-menu.selection-readout"
}

pub(super) fn collection_context_menu_duplicate_selected_label() -> &'static str {
    "Duplicate selected assets"
}

pub(super) fn collection_context_menu_rename_active_label() -> &'static str {
    "Rename active asset"
}

pub(super) fn collection_context_menu_delete_selected_label() -> &'static str {
    "Delete selected assets"
}

pub(super) fn collection_context_menu_dismiss_label() -> &'static str {
    "Dismiss quick actions"
}

pub(super) fn collection_context_menu_duplicate_selected_options(
    enabled: bool,
    close_popup: Model<bool>,
) -> kit::MenuItemOptions {
    collection_context_menu_action_options(
        enabled,
        close_popup,
        Some("Primary+D"),
        "imui-editor-proof.authoring.imui.collection.context-menu.duplicate-selected",
    )
}

pub(super) fn collection_context_menu_rename_active_options(
    enabled: bool,
    close_popup: Model<bool>,
) -> kit::MenuItemOptions {
    collection_context_menu_action_options(
        enabled,
        close_popup,
        Some("F2"),
        "imui-editor-proof.authoring.imui.collection.context-menu.rename",
    )
}

pub(super) fn collection_context_menu_delete_selected_options(
    enabled: bool,
    close_popup: Model<bool>,
) -> kit::MenuItemOptions {
    collection_context_menu_action_options(
        enabled,
        close_popup,
        Some("Del"),
        "imui-editor-proof.authoring.imui.collection.context-menu.delete-selected",
    )
}

pub(super) fn collection_context_menu_dismiss_options(
    close_popup: Model<bool>,
) -> kit::MenuItemOptions {
    collection_context_menu_action_options(
        true,
        close_popup,
        None,
        "imui-editor-proof.authoring.imui.collection.context-menu.dismiss",
    )
}

fn collection_context_menu_action_options(
    enabled: bool,
    close_popup: Model<bool>,
    shortcut: Option<&'static str>,
    test_id: &'static str,
) -> kit::MenuItemOptions {
    kit::MenuItemOptions {
        enabled,
        close_popup: Some(close_popup),
        shortcut: shortcut.map(Arc::from),
        test_id: Some(Arc::from(test_id)),
        ..Default::default()
    }
}
