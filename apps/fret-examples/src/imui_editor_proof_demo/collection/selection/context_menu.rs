use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::ProofCollectionKeyboardState;

pub(in super::super) fn proof_collection_context_menu_selection(
    selection: &ImUiMultiSelectState<Arc<str>>,
    asset_id: Arc<str>,
) -> (ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState) {
    let next_selection = if selection.is_selected(&asset_id) {
        selection.clone()
    } else {
        ImUiMultiSelectState::single(asset_id.clone())
    };

    (
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(asset_id),
        },
    )
}

#[cfg(test)]
mod tests;
