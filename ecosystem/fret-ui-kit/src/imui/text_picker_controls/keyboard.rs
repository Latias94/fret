use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

mod handler;

pub(super) use handler::install_picker_keyboard_handler;

#[derive(Debug, Clone)]
pub(super) struct InputTextPickerKeyboardPick {
    pub(super) source_index: usize,
    pub(super) value: Arc<str>,
}

#[derive(Debug, Clone, Default)]
pub(super) struct InputTextPickerKeyboardState {
    pub(super) active_source_index: Option<usize>,
    pub(super) active_element: Option<fret_ui::GlobalElementId>,
    pub(super) picked: Option<InputTextPickerKeyboardPick>,
}

#[derive(Debug, Clone)]
pub(super) struct InputTextPickerKeyboardSnapshot {
    pub(super) active_source_index: Option<usize>,
    pub(super) pending_pick: Option<InputTextPickerKeyboardPick>,
    pub(super) active_element: Option<fret_ui::GlobalElementId>,
}

pub(super) fn reconcile_picker_keyboard_state<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    state_model: &Model<InputTextPickerKeyboardState>,
    input_enabled_by_scope: bool,
    visible_candidates: &[(usize, Arc<str>)],
    hide_for_exact_match: bool,
) -> Option<InputTextPickerKeyboardSnapshot> {
    cx.app
        .models_mut()
        .update(state_model, |state| {
            let picked = state.picked.take();
            if !input_enabled_by_scope || visible_candidates.is_empty() || hide_for_exact_match {
                state.active_source_index = None;
                state.active_element = None;
            } else if let Some(active) = state.active_source_index
                && !visible_candidates
                    .iter()
                    .any(|(source_index, _)| *source_index == active)
            {
                state.active_source_index = None;
                state.active_element = None;
            } else if state.active_source_index.is_none() {
                state.active_element = None;
            }
            InputTextPickerKeyboardSnapshot {
                active_source_index: state.active_source_index,
                pending_pick: picked,
                active_element: state.active_element,
            }
        })
        .ok()
}
