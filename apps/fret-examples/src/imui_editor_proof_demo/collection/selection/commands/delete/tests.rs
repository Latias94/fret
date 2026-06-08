use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::super::super::super::{ProofCollectionAsset, authoring_parity_collection_assets};
use super::super::super::{ProofCollectionKeyboardState, proof_collection_assets_in_visible_order};
use super::proof_collection_delete_selection;

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
fn proof_collection_delete_selection_removes_selected_assets_and_refocuses_next_visible_item() {
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
        active_id: Some(Arc::from("stone-normal")),
    };

    let delete =
        proof_collection_delete_selection(&visible_assets, &stored_assets, &selection, &keyboard)
            .expect("delete should run when selected assets exist");

    assert_eq!(
        delete.deleted_ids,
        vec![Arc::from("stone-normal"), Arc::from("stone-orm")]
    );
    assert_eq!(
        delete
            .remaining_assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>(),
        vec![
            Arc::from("stone-albedo"),
            Arc::from("moss-overlay"),
            Arc::from("pebble-height"),
            Arc::from("dust-mask"),
        ]
    );
    assert_eq!(selected_ids(&delete.next_selection), vec!["moss-overlay"]);
    assert_eq!(anchor_id(&delete.next_selection), Some("moss-overlay"));
    assert_eq!(
        delete.next_keyboard.active_id,
        Some(Arc::from("moss-overlay"))
    );
}

#[test]
fn proof_collection_delete_selection_picks_previous_visible_item_at_end() {
    let stored_assets = authoring_parity_collection_assets()
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(stored_assets.clone()),
        false,
    );
    let selection = selection_state(&["dust-mask"], Some("dust-mask"));
    let keyboard = ProofCollectionKeyboardState {
        active_id: Some(Arc::from("dust-mask")),
    };

    let delete =
        proof_collection_delete_selection(&visible_assets, &stored_assets, &selection, &keyboard)
            .expect("delete should run when the tail item is selected");

    assert_eq!(delete.deleted_ids, vec![Arc::from("dust-mask")]);
    assert_eq!(selected_ids(&delete.next_selection), vec!["pebble-height"]);
    assert_eq!(anchor_id(&delete.next_selection), Some("pebble-height"));
    assert_eq!(
        delete.next_keyboard.active_id,
        Some(Arc::from("pebble-height"))
    );
}
