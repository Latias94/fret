use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::{ProofCollectionKeyboardState, proof_collection_active_id};

mod navigation;

use navigation::{proof_collection_keyboard_move_selection, proof_collection_keyboard_next_index};

pub(in super::super) fn proof_collection_keyboard_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    columns: usize,
    key: KeyCode,
    modifiers: Modifiers,
) -> Option<(ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState)> {
    if collection_keys.is_empty() || modifiers.alt || modifiers.ctrl || modifiers.meta {
        return None;
    }

    if key == KeyCode::Escape {
        return Some((
            ImUiMultiSelectState::default(),
            ProofCollectionKeyboardState {
                active_id: proof_collection_active_id(collection_keys, selection, keyboard),
            },
        ));
    }

    let current_id = proof_collection_active_id(collection_keys, selection, keyboard)?;
    let current_index = collection_keys
        .iter()
        .position(|item| item == &current_id)?;
    let next_index =
        proof_collection_keyboard_next_index(current_index, collection_keys.len(), columns, key)?;
    let next_id = collection_keys[next_index].clone();
    let next_selection = proof_collection_keyboard_move_selection(
        collection_keys,
        selection,
        next_id.clone(),
        modifiers.shift,
    );

    Some((
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(next_id),
        },
    ))
}

#[cfg(test)]
mod tests;
