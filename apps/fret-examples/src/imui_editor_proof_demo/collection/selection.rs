use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::ProofCollectionAsset;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProofCollectionKeyboardState {
    pub(super) active_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionDeleteResult {
    pub(super) remaining_assets: Vec<ProofCollectionAsset>,
    pub(super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(super) next_keyboard: ProofCollectionKeyboardState,
    pub(super) deleted_assets: Vec<ProofCollectionAsset>,
    pub(super) deleted_ids: Vec<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProofCollectionDuplicateResult {
    pub(super) next_assets: Vec<ProofCollectionAsset>,
    pub(super) duplicated_assets: Vec<ProofCollectionAsset>,
    pub(super) next_selection: ImUiMultiSelectState<Arc<str>>,
    pub(super) next_keyboard: ProofCollectionKeyboardState,
}

pub(super) fn proof_collection_assets_in_visible_order(
    assets: Arc<[ProofCollectionAsset]>,
    reverse_order: bool,
) -> Vec<ProofCollectionAsset> {
    let mut visible = assets.iter().cloned().collect::<Vec<_>>();
    if reverse_order {
        visible.reverse();
    }
    visible
}

pub(super) fn proof_collection_selected_assets<'a>(
    assets: &'a [ProofCollectionAsset],
    selection: &ImUiMultiSelectState<Arc<str>>,
) -> Vec<&'a ProofCollectionAsset> {
    let by_id = assets
        .iter()
        .map(|asset| (asset.id.as_ref(), asset))
        .collect::<HashMap<_, _>>();

    selection
        .selected()
        .iter()
        .filter_map(|id| by_id.get(id.as_ref()).copied())
        .collect()
}

pub(super) fn proof_collection_active_id(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
) -> Option<Arc<str>> {
    let contains = |id: &Arc<str>| collection_keys.iter().any(|key| key == id);

    keyboard
        .active_id
        .clone()
        .filter(contains)
        .or_else(|| selection.anchor().cloned().filter(contains))
        .or_else(|| selection.first_selected().cloned().filter(contains))
        .or_else(|| collection_keys.first().cloned())
}

pub(super) fn proof_collection_select_all_shortcut_matches(
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    matches!(key, KeyCode::KeyA)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

pub(super) fn proof_collection_duplicate_shortcut_matches(
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    matches!(key, KeyCode::KeyD)
        && !modifiers.alt
        && !modifiers.shift
        && (modifiers.ctrl || modifiers.meta)
}

pub(super) fn proof_collection_select_all_selection(
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

pub(super) fn proof_collection_context_menu_selection(
    selection: &ImUiMultiSelectState<Arc<str>>,
    asset_id: Arc<str>,
) -> (ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState) {
    let next_selection = if selection.is_selected(&asset_id) {
        selection.clone()
    } else {
        ImUiMultiSelectState::single(asset_id.clone())
    };

    (
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(asset_id),
        },
    )
}

pub(super) fn proof_collection_keyboard_next_index(
    current: usize,
    len: usize,
    columns: usize,
    key: KeyCode,
) -> Option<usize> {
    let last = len.checked_sub(1)?;
    match key {
        KeyCode::ArrowRight => Some((current + 1).min(last)),
        KeyCode::ArrowLeft => Some(current.saturating_sub(1)),
        KeyCode::ArrowDown => Some((current + columns).min(last)),
        KeyCode::ArrowUp => Some(current.saturating_sub(columns)),
        KeyCode::Home => Some(0),
        KeyCode::End => Some(last),
        _ => None,
    }
}

pub(super) fn proof_collection_keyboard_move_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    next_id: Arc<str>,
    extend_range: bool,
) -> ImUiMultiSelectState<Arc<str>> {
    if !extend_range {
        return ImUiMultiSelectState::single(next_id);
    }

    let anchor = selection
        .anchor()
        .cloned()
        .unwrap_or_else(|| next_id.clone());
    let Some(anchor_index) = collection_keys.iter().position(|key| key == &anchor) else {
        return ImUiMultiSelectState::single(next_id);
    };
    let Some(next_index) = collection_keys.iter().position(|key| key == &next_id) else {
        return ImUiMultiSelectState::single(next_id);
    };
    let (start, end) = if anchor_index <= next_index {
        (anchor_index, next_index)
    } else {
        (next_index, anchor_index)
    };

    ImUiMultiSelectState::from_ordered_selection(
        collection_keys,
        collection_keys[start..=end].to_vec(),
        Some(anchor),
    )
}

pub(super) fn proof_collection_keyboard_selection(
    collection_keys: &[Arc<str>],
    selection: &ImUiMultiSelectState<Arc<str>>,
    keyboard: &ProofCollectionKeyboardState,
    columns: usize,
    key: KeyCode,
    modifiers: Modifiers,
) -> Option<(ImUiMultiSelectState<Arc<str>>, ProofCollectionKeyboardState)> {
    if collection_keys.is_empty() || modifiers.alt || modifiers.ctrl || modifiers.meta {
        return None;
    }

    if key == KeyCode::Escape {
        return Some((
            ImUiMultiSelectState::default(),
            ProofCollectionKeyboardState {
                active_id: proof_collection_active_id(collection_keys, selection, keyboard),
            },
        ));
    }

    let current_id = proof_collection_active_id(collection_keys, selection, keyboard)?;
    let current_index = collection_keys
        .iter()
        .position(|item| item == &current_id)?;
    let next_index =
        proof_collection_keyboard_next_index(current_index, collection_keys.len(), columns, key)?;
    let next_id = collection_keys[next_index].clone();
    let next_selection = proof_collection_keyboard_move_selection(
        collection_keys,
        selection,
        next_id.clone(),
        modifiers.shift,
    );

    Some((
        next_selection,
        ProofCollectionKeyboardState {
            active_id: Some(next_id),
        },
    ))
}

pub(super) fn proof_collection_delete_key_matches(key: KeyCode) -> bool {
    matches!(key, KeyCode::Delete | KeyCode::Backspace)
}

pub(super) fn proof_collection_delete_selection(
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

pub(super) fn proof_collection_duplicate_selection(
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
    use super::super::authoring_parity_collection_assets;
    use super::super::geometry::PROOF_COLLECTION_GRID_FALLBACK_COLUMNS;
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
    fn proof_collection_keyboard_arrow_replaces_selection_and_moves_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowRight,
            Modifiers::default(),
        )
        .expect("plain arrow navigation should be handled");

        assert_eq!(selected_ids(&next_selection), vec!["stone-normal"]);
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-normal")));
    }

    #[test]
    fn proof_collection_keyboard_shift_navigation_extends_range_from_anchor() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-normal")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::ArrowDown,
            Modifiers {
                shift: true,
                ..Default::default()
            },
        )
        .expect("shift+arrow navigation should be handled");

        assert_eq!(
            selected_ids(&next_selection),
            vec!["stone-normal", "stone-orm", "moss-overlay", "pebble-height",]
        );
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("pebble-height")));
    }

    #[test]
    fn proof_collection_keyboard_escape_clears_selection_but_keeps_active_tile() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-orm")),
        };

        let (next_selection, next_keyboard) = proof_collection_keyboard_selection(
            &collection_keys,
            &selection,
            &keyboard,
            PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
            KeyCode::Escape,
            Modifiers::default(),
        )
        .expect("escape should be handled by the collection scope");

        assert!(next_selection.is_empty());
        assert_eq!(next_selection.anchor(), None);
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-orm")));
    }

    #[test]
    fn proof_collection_keyboard_ignores_primary_modifier_shortcuts() {
        let collection_keys = authoring_parity_collection_assets()
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection = selection_state(&["stone-albedo"], Some("stone-albedo"));
        let keyboard = ProofCollectionKeyboardState {
            active_id: Some(Arc::from("stone-albedo")),
        };

        assert!(
            proof_collection_keyboard_selection(
                &collection_keys,
                &selection,
                &keyboard,
                PROOF_COLLECTION_GRID_FALLBACK_COLUMNS,
                KeyCode::ArrowRight,
                Modifiers {
                    meta: true,
                    ..Default::default()
                },
            )
            .is_none(),
            "collection keyboard owner should stay app-local and avoid claiming primary-modifier shortcuts"
        );
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

    #[test]
    fn proof_collection_context_menu_selection_replaces_unselected_asset_and_sets_active_tile() {
        let selection = selection_state(&["stone-albedo", "stone-normal"], Some("stone-albedo"));

        let (next_selection, next_keyboard) =
            proof_collection_context_menu_selection(&selection, Arc::from("dust-mask"));

        assert_eq!(selected_ids(&next_selection), vec!["dust-mask"]);
        assert_eq!(anchor_id(&next_selection), Some("dust-mask"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("dust-mask")));
    }

    #[test]
    fn proof_collection_context_menu_selection_preserves_selected_range_and_updates_active_tile() {
        let selection = selection_state(&["stone-normal", "stone-orm"], Some("stone-normal"));

        let (next_selection, next_keyboard) =
            proof_collection_context_menu_selection(&selection, Arc::from("stone-orm"));

        assert_eq!(
            selected_ids(&next_selection),
            vec!["stone-normal", "stone-orm"]
        );
        assert_eq!(anchor_id(&next_selection), Some("stone-normal"));
        assert_eq!(next_keyboard.active_id, Some(Arc::from("stone-orm")));
    }
}
