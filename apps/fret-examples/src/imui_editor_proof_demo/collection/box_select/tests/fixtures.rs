use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

pub(super) fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
    selection.selected().iter().map(|id| id.as_ref()).collect()
}

pub(super) fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
    selection.anchor().map(|id| id.as_ref())
}
