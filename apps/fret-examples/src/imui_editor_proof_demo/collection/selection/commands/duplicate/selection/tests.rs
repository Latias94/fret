use std::sync::Arc;

use super::super::super::super::super::{ProofCollectionAsset, authoring_parity_collection_assets};
use super::super::super::super::{
    ProofCollectionKeyboardState, proof_collection_assets_in_visible_order,
};
use super::proof_collection_duplicate_selection_result;

mod fixtures;

use fixtures::{anchor_id, selected_ids, selection_state};

#[test]
fn proof_collection_duplicate_selection_reselects_visible_copies_and_preserves_active_copy() {
    let stored_assets = authoring_parity_collection_assets()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
        false,
    );
    let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
    let keyboard = ProofCollectionKeyboardState {
        active_id: Some(Arc::from("stone-orm")),
    };

    let duplicate = proof_collection_duplicate_selection_result(
        &visible_assets,
        &stored_assets,
        &selection,
        &keyboard,
        false,
    )
    .expect("duplicate should run when selected assets exist");

    assert_eq!(
        duplicate
            .duplicated_assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>(),
        vec![Arc::from("stone-normal-copy"), Arc::from("stone-orm-copy")]
    );
    assert_eq!(
        duplicate
            .duplicated_assets
            .iter()
            .map(|asset| asset.label.clone())
            .collect::<Vec<_>>(),
        vec![Arc::from("Stone Normal Copy"), Arc::from("Stone ORM Copy")]
    );
    assert_eq!(
        duplicate
            .next_assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>(),
        vec![
            Arc::from("stone-albedo"),
            Arc::from("stone-normal"),
            Arc::from("stone-normal-copy"),
            Arc::from("stone-orm"),
            Arc::from("stone-orm-copy"),
            Arc::from("moss-overlay"),
            Arc::from("pebble-height"),
            Arc::from("dust-mask"),
        ]
    );
    assert_eq!(
        selected_ids(&duplicate.next_selection),
        vec!["stone-normal-copy", "stone-orm-copy"]
    );
    assert_eq!(
        anchor_id(&duplicate.next_selection),
        Some("stone-normal-copy")
    );
    assert_eq!(
        duplicate.next_keyboard.active_id,
        Some(Arc::from("stone-orm-copy"))
    );
}

#[test]
fn proof_collection_duplicate_selection_uses_unique_copy_suffixes_when_copy_exists() {
    let mut stored_assets = authoring_parity_collection_assets()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    stored_assets.push(ProofCollectionAsset {
        id: Arc::from("stone-normal-copy"),
        label: Arc::from("Stone Normal Copy"),
        path: Arc::from("textures/stone/normal-copy.ktx2"),
        kind: Arc::from("Texture"),
        size_kib: 384,
    });
    let visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
        false,
    );
    let selection = selection_state(&["stone-normal"], Some("stone-normal"));
    let keyboard = ProofCollectionKeyboardState {
        active_id: Some(Arc::from("stone-normal")),
    };

    let duplicate = proof_collection_duplicate_selection_result(
        &visible_assets,
        &stored_assets,
        &selection,
        &keyboard,
        false,
    )
    .expect("duplicate should generate a unique copy even when one already exists");

    assert_eq!(
        duplicate
            .duplicated_assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>(),
        vec![Arc::from("stone-normal-copy-2")]
    );
    assert_eq!(
        duplicate
            .duplicated_assets
            .iter()
            .map(|asset| asset.label.clone())
            .collect::<Vec<_>>(),
        vec![Arc::from("Stone Normal Copy 2")]
    );
    assert_eq!(
        duplicate
            .duplicated_assets
            .iter()
            .map(|asset| asset.path.clone())
            .collect::<Vec<_>>(),
        vec![Arc::from("textures/stone/normal-copy-2.ktx2")]
    );
}
