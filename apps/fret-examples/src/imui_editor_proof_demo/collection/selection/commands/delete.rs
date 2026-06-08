use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::KeyCode;

use super::super::super::ProofCollectionAsset;
use super::super::{ProofCollectionKeyboardState, proof_collection_active_id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super::super) struct ProofCollectionDeleteResult {
    pub(in super::super::super) remaining_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(in super::super::super) next_keyboard: ProofCollectionKeyboardState,
    pub(in super::super::super) deleted_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) deleted_ids: Vec<Arc<str>>,
}

pub(in super::super::super) fn proof_collection_delete_key_matches(key: KeyCode) -> bool {
    matches!(key, KeyCode::Delete | KeyCode::Backspace)
}

pub(in super::super::super) fn proof_collection_delete_selection(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<ProofCollectionDeleteResult> {
    let deleted_assets = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let deleted_ids = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    if deleted_ids.is_empty() {
        return None;
    }

    let deleted_contains = |id: &Arc<str>| deleted_ids.iter().any(|item| item == id);
    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let focus_source_index = proof_collection_active_id(&visible_keys, selection, keyboard)
        .and_then(|id| visible_keys.iter().position(|key| key == &id))
        .or_else(|| {
            deleted_ids
                .last()
                .and_then(|id| visible_keys.iter().position(|key| key == id))
        })
        .unwrap_or(0);

    let remaining_visible = visible_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let remaining_assets = stored_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let next_active = if remaining_visible.is_empty() {
        None
    } else {
        Some(
            remaining_visible[focus_source_index.min(remaining_visible.len() - 1)]
                .id
                .clone(),
        )
    };
    let next_selection = next_active
        .clone()
        .map(ImUiMultiSelectState::single)
        .unwrap_or_default();

    Some(ProofCollectionDeleteResult {
        remaining_assets,
        next_selection,
        next_keyboard: ProofCollectionKeyboardState {
            active_id: next_active,
        },
        deleted_assets,
        deleted_ids,
    })
}

#[cfg(test)]
mod tests;
