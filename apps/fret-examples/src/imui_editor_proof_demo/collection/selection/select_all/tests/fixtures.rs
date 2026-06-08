use std::sync::Arc;

use fret::imui::kit::ImUiMultiSelectState;

pub(super) fn selection_state(
    selected: &[&str],
    anchor: Option<&str>,
) -> ImUiMultiSelectState<Arc<str>> {
    ImUiMultiSelectState::new(
        selected.iter().map(|id| Arc::from(*id)).collect(),
        anchor.map(Arc::from),
    )
}

pub(super) fn anchor_id(selection: &ImUiMultiSelectState<Arc<str>>) -> Option<&str> {
    selection.anchor().map(|id| id.as_ref())
}
