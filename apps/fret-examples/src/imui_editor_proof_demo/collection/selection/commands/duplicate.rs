use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::super::super::ProofCollectionAsset;
use super::super::ProofCollectionKeyboardState;

mod naming;
mod selection;

use naming::ProofCollectionDuplicateNameRegistry;
use selection::proof_collection_duplicate_selection_result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super::super) struct ProofCollectionDuplicateResult {
    pub(in super::super::super) next_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) duplicated_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(in super::super::super) next_keyboard: ProofCollectionKeyboardState,
}

pub(in super::super::super) fn proof_collection_duplicate_shortcut_matches(
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    matches!(key, KeyCode::KeyD)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

pub(in super::super::super) fn proof_collection_duplicate_selection(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    reverse_order: bool,
) -> Option<ProofCollectionDuplicateResult> {
    proof_collection_duplicate_selection_result(
        visible_assets,
        stored_assets,
        selection,
        keyboard,
        reverse_order,
    )
}

#[cfg(test)]
mod tests;
