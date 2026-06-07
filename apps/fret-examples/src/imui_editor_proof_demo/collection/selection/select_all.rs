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
mod tests {
    use std::sync::Arc;

    use fret::imui::kit::ImUiMultiSelectState;

    use super::super::ProofCollectionKeyboardState;
    use super::*;

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
        selection.anchor().map(|id| id.as_ref())
    }

    #[test]
    fn proof_collection_select_all_selection_uses_visible_order_and_preserves_active_tile() {
        let collection_keys = vec![
            Arc::from("dust-mask"),
            Arc::from("pebble-height"),
            Arc::from("moss-overlay"),
        ];
        let selection = selection_state(&["moss-overlay"], Some("moss-overlay"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("pebble-height")),
        };

        let (next_selection, next_keyboard) =
            proof_collection_select_all_selection(&collection_keys, &selection, &keyboard)
                .expect("select-all should run when visible assets exist");

        assert_eq!(next_selection.selected(), collection_keys.as_slice());
        assert_eq!(anchor_id(&next_selection), Some("moss-overlay"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("pebble-height")));
    }

    #[test]
    fn proof_collection_select_all_selection_falls_back_to_first_visible_asset() {
        let collection_keys = vec![Arc::from("stone-albedo"), Arc::from("stone-normal")];
        let selection = ImUiMultiSelectState::default();
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("missing")),
        };

        let (next_selection, next_keyboard) =
            proof_collection_select_all_selection(&collection_keys, &selection, &keyboard)
                .expect("select-all should fall back to the first visible asset");

        assert_eq!(next_selection.selected(), collection_keys.as_slice());
        assert_eq!(anchor_id(&next_selection), Some("stone-albedo"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-albedo")));
    }

    #[test]
    fn proof_collection_select_all_shortcut_matches_primary_a_only() {
        assert!(proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        ));
        assert!(proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers::default(),
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        ));
        assert!(!proof_collection_select_all_shortcut_matches(
            KeyCode::KeyA,
            Modifiers {
                ctrl: true,
                alt: true,
                ..Default::default()
            },
        ));
    }
}
