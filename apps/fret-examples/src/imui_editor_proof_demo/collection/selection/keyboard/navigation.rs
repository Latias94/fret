use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;
use fret_core::KeyCode;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Vec<Arc<str>> {
        ["stone-albedo", "stone-normal", "stone-orm", "moss-overlay"]
            .into_iter()
            .map(Arc::from)
            .collect()
    }

    fn selection_state(selected: &[&str], anchor: Option<&str>) -> ImUiMultiSelectState<Arc<str>> {
        ImUiMultiSelectState::new(
            selected.iter().map(|id| Arc::from(*id)).collect(),
            anchor.map(Arc::from),
        )
    }

    fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
        selection.selected().iter().map(|id| id.as_ref()).collect()
    }

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
}
