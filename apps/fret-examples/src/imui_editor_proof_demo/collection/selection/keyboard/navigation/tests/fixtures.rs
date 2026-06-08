use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

pub(super) fn keys() -> Vec<Arc<str>> {
    ["stone-albedo", "stone-normal", "stone-orm", "moss-overlay"]
        .into_iter()
        .map(Arc::from)
        .collect()
}

pub(super) fn selection_state(
    selected: &[&str],
    anchor: Option<&str>,
) -> ImUiMultiSelectState<Arc<str>> {
    ImUiMultiSelectState::new(
        selected.iter().map(|id| Arc::from(*id)).collect(),
        anchor.map(Arc::from),
    )
}

pub(super) fn selected_ids(selection: &ImUiMultiSelectState<Arc<str>>) -> Vec<&str> {
    selection.selected().iter().map(|id| id.as_ref()).collect()
}
