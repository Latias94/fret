use super::*;

mod fixtures;

use fixtures::{keys, selected_ids, selection_state};

#[test]
fn proof_collection_keyboard_next_index_moves_with_columns_and_edges() {
    assert_eq!(
        proof_collection_keyboard_next_index(0, 4, 2, KeyCode::ArrowRight),
        Some(1)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(0, 4, 2, KeyCode::ArrowDown),
        Some(2)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(3, 4, 2, KeyCode::ArrowRight),
        Some(3)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(0, 4, 2, KeyCode::ArrowLeft),
        Some(0)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(2, 4, 2, KeyCode::Home),
        Some(0)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(1, 4, 2, KeyCode::End),
        Some(3)
    );
    assert_eq!(
        proof_collection_keyboard_next_index(1, 0, 2, KeyCode::ArrowRight),
        None
    );
}

#[test]
fn proof_collection_keyboard_move_selection_extends_from_anchor_in_collection_order() {
    let keys = keys();
    let selection = selection_state(&["stone-normal"], Some("stone-normal"));

    let next_selection = proof_collection_keyboard_move_selection(
        &keys,
        &selection,
        Arc::from("moss-overlay"),
        true,
    );

    assert_eq!(
        selected_ids(&next_selection),
        vec!["stone-normal", "stone-orm", "moss-overlay"]
    );
    assert_eq!(next_selection.anchor(), Some(&Arc::from("stone-normal")));
}
