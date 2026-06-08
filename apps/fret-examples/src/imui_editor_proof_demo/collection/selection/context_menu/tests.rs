use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

use super::proof_collection_context_menu_selection;

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
