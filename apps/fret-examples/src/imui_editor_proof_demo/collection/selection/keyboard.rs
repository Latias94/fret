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
mod tests {
    use std::sync::Arc;

    use fret::imui::kit::ImUiMultiSelectState;

    use super::super::super::authoring_parity_collection_assets;
    use super::super::super::geometry::PROOF_COLLECTION_GRID_FALLBACK_COLUMNS;
    use super::*;

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
        selection.selected().iter().map(|id| id.as_ref()).collect()
    }

    fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
        selection.anchor().map(|id| id.as_ref())
    }

    #[test]
    fn proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowRight,
            Modifiers::default(),
        )
        .expect("plain arrow navigation should be handled");

        assert_eq!(selected_ids(&next_selection), vec!["stone-normal"]);
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-normal")));
    }

    #[test]
    fn proof_collection_keyboard_shift_navigation_extends_range_from_anchor() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowDown,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        )
        .expect("shift+arrow navigation should be handled");

        assert_eq!(
            selected_ids(&next_selection),
            vec!["stone-normal", "stone-orm", "moss-overlay", "pebble-height",]
        );
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("pebble-height")));
    }

    #[test]
    fn proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-orm")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::Escape,
            Modifiers::default(),
        )
        .expect("escape should be handled by the collection scope");

        assert!(next_selection.is_empty());
        assert_eq!(next_selection.anchor(), None);
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-orm")));
    }

    #[test]
    fn proof_collection_keyboard_ignores_primary_modifier_shortcuts() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        assert!(
            proof_collection_keyboard_selection(
                &collection_keys,
                &selection,
                &keyboard,
                PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
                KeyCode::ArrowRight,
                Modifiers {
                    meta: true,
                    ..Default::default()
                },
            )
            .is_none(),
            "collection keyboard owner should stay app-local and avoid claiming primary-modifier shortcuts"
        );
    }
}
