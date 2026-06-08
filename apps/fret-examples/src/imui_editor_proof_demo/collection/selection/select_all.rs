use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::{ProofCollectionKeyboardState, proof_collection_active_id};

pub(in super::super) fn proof_collection_select_all_shortcut_matches(
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    matches!(key, KeyCode::KeyA)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

pub(in super::super) fn proof_collection_select_all_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<(ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState)> {
    let contains = |id: &Arc<str>| collection_keys.iter().any(|key| key == id);
    let next_active = proof_collection_active_id(collection_keys, selection, keyboard)?;
    let next_anchor = selection
        .anchor()
        .cloned()
        .filter(contains)
        .or_else(|| collection_keys.first().cloned());

    Some((
        ImUiMultiSelectState::from_ordered_selection(
            collection_keys,
            collection_keys.to_vec(),
            next_anchor,
        ),
        ProofCollectionKeyboardState {
            active_id: Some(next_active),
        },
    ))
}

#[cfg(test)]
mod tests;
