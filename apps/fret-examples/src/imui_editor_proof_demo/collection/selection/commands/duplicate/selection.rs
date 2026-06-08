use std::collections::HashMap;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::super::super::super::ProofCollectionAsset;
use super::super::super::{
    ProofCollectionKeyboardState, proof_collection_active_id,
    proof_collection_assets_in_visible_order,
};
use super::{ProofCollectionDuplicateNameRegistry, ProofCollectionDuplicateResult};

pub(super) fn proof_collection_duplicate_selection_result(
    visible_assets: &[ProofCollectionAsset],
    stored_assets: &[ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    reverse_order: bool,
) -> Option<ProofCollectionDuplicateResult> {
    let selected_visible_assets = visible_assets
        .iter()
        .filter(|asset| selection.is_selected(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    if selected_visible_assets.is_empty() {
        return None;
    }

    let visible_keys = visible_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let active_id = proof_collection_active_id(&visible_keys, selection, keyboard);
    let mut name_registry = ProofCollectionDuplicateNameRegistry::from_assets(stored_assets);
    let mut duplicates_by_source = HashMap::<Arc<str>, ProofCollectionAsset>::new();

    for asset in &selected_visible_assets {
        let duplicate = ProofCollectionAsset {
            id: name_registry.duplicate_id(asset.id.as_ref()),
            label: name_registry.duplicate_label(asset.label.as_ref()),
            path: name_registry.duplicate_path(asset.path.as_ref()),
            kind: asset.kind.clone(),
            size_kib: asset.size_kib,
        };
        duplicates_by_source.insert(asset.id.clone(), duplicate);
    }

    let mut remaining_and_duplicates =
        Vec::with_capacity(stored_assets.len() + duplicates_by_source.len());
    for asset in stored_assets {
        remaining_and_duplicates.push(asset.clone());
        if let Some(duplicate) = duplicates_by_source.get(&asset.id) {
            remaining_and_duplicates.push(duplicate.clone());
        }
    }

    let next_visible_assets = proof_collection_assets_in_visible_order(
        Arc::<[ProofCollectionAsset]>::from(remaining_and_duplicates.clone()),
        reverse_order,
    );
    let duplicate_ids = duplicates_by_source
        .values()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let duplicated_assets = next_visible_assets
        .iter()
        .filter(|asset| duplicate_ids.contains(&asset.id))
        .cloned()
        .collect::<Vec<_>>();
    let duplicated_ids = duplicated_assets
        .iter()
        .map(|asset| asset.id.clone())
        .collect::<Vec<_>>();
    let next_active = active_id
        .and_then(|id| duplicates_by_source.get(&id).map(|asset| asset.id.clone()))
        .or_else(|| duplicated_ids.first().cloned());
    let next_selection = duplicated_ids
        .first()
        .cloned()
        .map(|anchor| ImUiMultiSelectState::new(duplicated_ids.clone(), Some(anchor)))
        .unwrap_or_default();

    Some(ProofCollectionDuplicateResult {
        next_assets: remaining_and_duplicates,
        duplicated_assets,
        next_selection,
        next_keyboard: ProofCollectionKeyboardState {
            active_id: next_active,
        },
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret::imui::kit::ImUiMultiSelectState;

    use super::super::super::super::super::{
        ProofCollectionAsset, authoring_parity_collection_assets,
    };
    use super::super::super::super::{
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
}
