use fret_core::{KeyCode, Modifiers};

use super::proof_collection_duplicate_shortcut_matches;

#[test]
fn proof_collection_duplicate_shortcut_matches_primary_d_only() {
    assert!(proof_collection_duplicate_shortcut_matches(
        KeyCode::KeyD,
        Modifiers {
            meta: true,
            ..Default::default()
        },
    ));
    assert!(proof_collection_duplicate_shortcut_matches(
        KeyCode::KeyD,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    ));
    assert!(!proof_collection_duplicate_shortcut_matches(
        KeyCode::KeyD,
        Modifiers::default(),
    ));
    assert!(!proof_collection_duplicate_shortcut_matches(
        KeyCode::KeyD,
        Modifiers {
            shift: true,
            meta: true,
            ..Default::default()
        },
    ));
    assert!(!proof_collection_duplicate_shortcut_matches(
        KeyCode::KeyD,
        Modifiers {
            alt: true,
            ctrl: true,
            ..Default::default()
        },
    ));
}
