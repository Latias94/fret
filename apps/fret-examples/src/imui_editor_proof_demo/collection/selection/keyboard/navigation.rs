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
mod tests;
