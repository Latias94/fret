use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::ProofCollectionAsset;
use super::selection::{
    ProofCollectionKeyboardState, proof_collection_active_id, proof_collection_selected_assets,
};

mod status;

pub(super) use status::{
    proof_collection_command_status_line, proof_collection_delete_status,
    proof_collection_duplicate_status, proof_collection_rename_cancel_status,
    proof_collection_rename_commit_status, proof_collection_rename_invalid_status,
    proof_collection_rename_ready_status, proof_collection_rename_status_line,
    proof_collection_select_all_status,
};

pub(super) fn proof_collection_selection_line(
    assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> String {
    let selected = proof_collection_selected_assets(assets, selection);
    if selected.is_empty() {
        return "Selection: none. Click to select, primary-modifier click to toggle, shift-click to extend, arrow/home/end to move the active tile, or drag background to box-select.".to_string();
    }

    let labels = selected
        .iter()
        .map(|asset| asset.label.as_ref())
        .collect::<Vec<_>>()
        .join(", ");
    format!("Selection: {} asset(s) | {labels}", selected.len())
}

pub(super) fn proof_collection_visible_order_line(assets: &[ProofCollectionAsset]) -> String {
    let labels = assets
        .iter()
        .map(|asset| asset.label.as_ref())
        .collect::<Vec<_>>()
        .join(" -> ");
    format!("Visible order: {labels}")
}

pub(super) fn proof_collection_active_line(
    assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> String {
    let visible_keys = assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&visible_keys, selection, keyboard);
    let Some(active_id) = active_id else {
        return "Active tile: none. Click background to focus the collection scope, then use Arrow/Home/End to drive selection app-locally.".to_string();
    };
    let Some(asset) = assets.iter().find(|asset| asset.id == active_id) else {
        return "Active tile: none. Click background to focus the collection scope, then use Arrow/Home/End to drive selection app-locally.".to_string();
    };

    format!(
        "Active tile: {}. Shift+Arrow/Home/End extends from the current anchor; Escape clears the selection without widening shared IMUI helper ownership.",
        asset.label
    )
}

pub(super) fn proof_collection_assets_line(assets: &[ProofCollectionAsset]) -> String {
    format!(
        "Assets: {}. Press Delete/Backspace or use the explicit action button to remove the selected set app-locally.",
        assets.len()
    )
}

pub(super) fn proof_collection_command_package_line() -> String {
    "Duplicate, delete, rename, and select-all stay inside one app-owned collection command package; duplicate/delete/rename now route across keyboard, explicit buttons, and context menu without widening shared IMUI helpers.".to_string()
}

pub(super) fn proof_collection_select_all_line() -> String {
    "Primary+A selects all visible assets inside the focused collection scope.".to_string()
}

pub(super) fn proof_collection_rename_line() -> String {
    "F2, the explicit rename button, or the context menu starts an app-local inline rename editor for the current active asset.".to_string()
}

pub(super) fn proof_collection_context_menu_line() -> String {
    "Right-click an asset or the collection background to open app-local collection actions."
        .to_string()
}
