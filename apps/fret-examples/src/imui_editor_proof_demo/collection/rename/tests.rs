use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};

use super::super::authoring_parity_collection_assets;
use super::super::selection::ProofCollectionKeyboardState;
use super::{proof_collection_begin_rename_session, proof_collection_rename_shortcut_matches};

mod fixtures;

use fixtures::selection_state;

#[test]
fn proof_collection_begin_rename_session_prefers_active_visible_asset() {
    let visible_assets = authoring_parity_collection_assets();
    let selection = selection_state(&["stone-albedo", "stone-normal"], Some("stone-albedo"));
    let keyboard = ProofCollectionKeyboardState {
        active_id: Some(Arc::from("stone-normal")),
    };

    let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
        .expect("rename should target the active visible asset");

    assert_eq!(session.target_id, Arc::from("stone-normal"));
    assert_eq!(session.original_label, Arc::from("Stone Normal"));
}

#[test]
fn proof_collection_begin_rename_session_falls_back_to_first_visible_asset() {
    let visible_assets = authoring_parity_collection_assets();
    let selection = selection_state(&[], None);
    let keyboard = ProofCollectionKeyboardState {
        active_id: Some(Arc::from("missing")),
    };

    let session = proof_collection_begin_rename_session(&visible_assets, &selection, &keyboard)
        .expect("rename should fall back to the first visible asset");

    assert_eq!(session.target_id, Arc::from("stone-albedo"));
    assert_eq!(session.original_label, Arc::from("Stone Albedo"));
}

#[test]
fn proof_collection_rename_shortcut_matches_plain_f2_only() {
    assert!(proof_collection_rename_shortcut_matches(
        KeyCode::F2,
        Modifiers::default(),
    ));
    assert!(!proof_collection_rename_shortcut_matches(
        KeyCode::F2,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    ));
    assert!(!proof_collection_rename_shortcut_matches(
        KeyCode::F2,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    ));
    assert!(!proof_collection_rename_shortcut_matches(
        KeyCode::KeyA,
        Modifiers::default(),
    ));
}
