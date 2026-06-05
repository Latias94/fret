use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::super::ProofCollectionAsset;
use super::{
    ProofCollectionKeyboardState, proof_collection_active_id,
    proof_collection_assets_in_visible_order,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProofCollectionDeleteResult {
    pub(in super::super) remaining_assets: Vec<ProofCollectionAsset>,
    pub(in super::super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(in super::super) next_keyboard: ProofCollectionKeyboardState,
    pub(in super::super) deleted_assets: Vec<ProofCollectionAsset>,
    pub(in super::super) deleted_ids: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in super::super) struct ProofCollectionDuplicateResult {
    pub(in super::super) next_assets: Vec<ProofCollectionAsset>,
    pub(in super::super) duplicated_assets: Vec<ProofCollectionAsset>,
    pub(in super::super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(in super::super) next_keyboard: ProofCollectionKeyboardState,
}

pub(in super::super) fn proof_collection_duplicate_shortcut_matches(
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    matches!(key, KeyCode::KeyD)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

pub(in super::super) fn proof_collection_delete_key_matches(key: KeyCode) -> bool {
    matches!(key, KeyCode::Delete | KeyCode::Backspace)
}

pub(in super::super) fn proof_collection_delete_selection(
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

fn proof_collection_duplicate_label_candidate(label: &str, index: usize) -> String {
    if index == 1 {
        format!("{label} Copy")
    } else {
        format!("{label} Copy {index}")
    }
}

fn proof_collection_duplicate_id_candidate(id: &str, index: usize) -> String {
    if index == 1 {
        format!("{id}-copy")
    } else {
        format!("{id}-copy-{index}")
    }
}

fn proof_collection_duplicate_path_candidate(path: &str, index: usize) -> String {
    let suffix = if index == 1 {
        "-copy".to_string()
    } else {
        format!("-copy-{index}")
    };

    match path.rsplit_once('.') {
        Some((stem, ext)) if !ext.contains('/') => format!("{stem}{suffix}.{ext}"),
        _ => format!("{path}{suffix}"),
    }
}

fn proof_collection_unique_copy_text(
    used: &mut HashSet<String>,
    candidate: impl Fn(usize) -> String,
) -> Arc<str> {
    let mut index = 1;
    loop {
        let value = candidate(index);
        if used.insert(value.clone()) {
            return Arc::from(value);
        }
        index += 1;
    }
}

pub(in super::super) fn proof_collection_duplicate_selection(
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
    let mut used_ids = stored_assets
        .iter()
        .map(|asset| asset.id.to_string())
        .collect::<HashSet<_>>();
    let mut used_labels = stored_assets
        .iter()
        .map(|asset| asset.label.to_string())
        .collect::<HashSet<_>>();
    let mut used_paths = stored_assets
        .iter()
        .map(|asset| asset.path.to_string())
        .collect::<HashSet<_>>();
    let mut duplicates_by_source = HashMap::<Arc<str>, ProofCollectionAsset>::new();

    for asset in &selected_visible_assets {
        let duplicate = ProofCollectionAsset {
            id: proof_collection_unique_copy_text(&mut used_ids, |index| {
                proof_collection_duplicate_id_candidate(asset.id.as_ref(), index)
            }),
            label: proof_collection_unique_copy_text(&mut used_labels, |index| {
                proof_collection_duplicate_label_candidate(asset.label.as_ref(), index)
            }),
            path: proof_collection_unique_copy_text(&mut used_paths, |index| {
                proof_collection_duplicate_path_candidate(asset.path.as_ref(), index)
            }),
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
    let duplicated_ids_set = duplicates_by_source
        .values()
        .map(|asset| asset.id.as_ref())
        .collect::<HashSet<_>>();
    let duplicated_assets = next_visible_assets
        .iter()
        .filter(|asset| duplicated_ids_set.contains(asset.id.as_ref()))
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
    use super::super::super::{ProofCollectionAsset, authoring_parity_collection_assets};
    use super::super::{ProofCollectionKeyboardState, proof_collection_assets_in_visible_order};
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

        let duplicate = proof_collection_duplicate_selection(
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

        let duplicate = proof_collection_duplicate_selection(
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
