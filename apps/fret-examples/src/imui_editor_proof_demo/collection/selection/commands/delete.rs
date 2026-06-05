use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::KeyCode;

use super::super::super::ProofCollectionAsset;
use super::super::{ProofCollectionKeyboardState, proof_collection_active_id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super::super) struct ProofCollectionDeleteResult {
    pub(in super::super::super) remaining_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(in super::super::super) next_keyboard: ProofCollectionKeyboardState,
    pub(in super::super::super) deleted_assets: Vec<ProofCollectionAsset>,
    pub(in super::super::super) deleted_ids: Vec<Arc<str>>,
}

pub(in super::super::super) fn proof_collection_delete_key_matches(key: KeyCode) -> bool {
    matches!(key, KeyCode::Delete | KeyCode::Backspace)
}

pub(in super::super::super) fn proof_collection_delete_selection(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<ProofCollectionDeleteResult> {
    let deleted_assets = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let deleted_ids = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    if deleted_ids.is_empty() {
        return None;
    }

    let deleted_contains = |id: &Arc<str>| deleted_ids.iter().any(|item| item == id);
    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let focus_source_index = proof_collection_active_id(&visible_keys, selection, keyboard)
        .and_then(|id| visible_keys.iter().position(|key| key == &id))
        .or_else(|| {
            deleted_ids
                .last()
                .and_then(|id| visible_keys.iter().position(|key| key == id))
        })
        .unwrap_or(0);

    let remaining_visible = visible_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let remaining_assets = stored_assets
        .iter()
        .filter(|asset| !deleted_contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let next_active = if remaining_visible.is_empty() {
        None
    } else {
        Some(
            remaining_visible[focus_source_index.min(remaining_visible.len() - 1)]
                .id
                .clone(),
        )
    };
    let next_selection = next_active
        .clone()
        .map(ImUiMultiSelectState::single)
        .unwrap_or_default();

    Some(ProofCollectionDeleteResult {
        remaining_assets,
        next_selection,
        next_keyboard: ProofCollectionKeyboardState {
            active_id: next_active,
        },
        deleted_assets,
        deleted_ids,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret::imui::kit::ImUiMultiSelectState;

    use super::super::super::super::{ProofCollectionAsset, authoring_parity_collection_assets};
    use super::super::super::{
        ProofCollectionKeyboardState, proof_collection_assets_in_visible_order,
    };
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

        let delete = proof_collection_delete_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
        )
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

        let delete = proof_collection_delete_selection(
            &visible_assets,
            &stored_assets,
            &selection,
            &keyboard,
        )
        .expect("delete should run when the tail item is selected");

        assert_eq!(delete.deleted_ids, vec![Arc::from("dust-mask")]);
        assert_eq!(selected_ids(&delete.next_selection), vec!["pebble-height"]);
        assert_eq!(anchor_id(&delete.next_selection), Some("pebble-height"));
        assert_eq!(
            delete.next_keyboard.active_id,
            Some(Arc::from("pebble-height"))
        );
    }
}
