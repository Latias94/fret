use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::super::ProofCollectionKeyboardState;
use super::{proof_collection_select_all_selection, proof_collection_select_all_shortcut_matches};

mod fixtures;

use fixtures::{anchor_id, selection_state};

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
