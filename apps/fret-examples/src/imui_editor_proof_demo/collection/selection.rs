use std::collections::HashMap;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::ProofCollectionAsset;

mod commands;
mod context_menu;
mod keyboard;
mod select_all;

pub(super) use commands::{
    ProofCollectionDeleteResult, ProofCollectionDuplicateResult,
    proof_collection_delete_key_matches, proof_collection_delete_selection,
    proof_collection_duplicate_selection, proof_collection_duplicate_shortcut_matches,
};
pub(super) use context_menu::proof_collection_context_menu_selection;
pub(super) use keyboard::proof_collection_keyboard_selection;
pub(super) use select_all::{
    proof_collection_select_all_selection, proof_collection_select_all_shortcut_matches,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProofCollectionKeyboardState {
    pub(super) active_id: Option<Arc<str>>,
}

pub(super) fn proof_collection_assets_in_visible_order(
    assets: Arc<[ProofCollectionAsset]>,
    reverse_order: bool,
) -> Vec<ProofCollectionAsset> {
    let mut visible = assets.iter().cloned().collect::<Vec<_>>();
    if reverse_order {
        visible.reverse();
    }
    visible
}

pub(super) fn proof_collection_selected_assets<'a>(
    assets: &'a [ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> Vec<&'a ProofCollectionAsset> {
    let by_id = assets
        .iter()
        .map(|asset| (asset.id.as_ref(), asset))
        .collect::<HashMap<_, _>>();

    selection
        .selected()
        .iter()
        .filter_map(|id| by_id.get(id.as_ref()).copied())
        .collect()
}

pub(super) fn proof_collection_active_id(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<Arc<str>> {
    let contains = |id: &Arc<str>| collection_keys.iter().any(|key| key == id);

    keyboard
        .active_id
        .clone()
        .filter(contains)
        .or_else(|| selection.anchor().cloned().filter(contains))
        .or_else(|| selection.first_selected().cloned().filter(contains))
        .or_else(|| collection_keys.first().cloned())
}
