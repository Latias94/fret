use std::collections::HashMap;
use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::{KeyCode, Modifiers};

use super::ProofCollectionAsset;

mod commands;
mod select_all;

pub(super) use commands::{
    ProofCollectionDeleteResult, ProofCollectionDuplicateResult,
    proof_collection_delete_key_matches, proof_collection_delete_selection,
    proof_collection_duplicate_selection, proof_collection_duplicate_shortcut_matches,
};
pub(super) use select_all::{
    proof_collection_select_all_selection, proof_collection_select_all_shortcut_matches,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProofCollectionKeyboardState {
    pub(super) active_id: Option<Arc<str>>,
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
