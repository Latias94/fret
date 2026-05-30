use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost, UiActionHostExt as _};

use super::super::InputTextPickerKeyboardState;

pub(super) fn move_picker_highlight(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    forward: bool,
    state: &Model<InputTextPickerKeyboardState>,
    visible_candidates: &[(usize, Arc<str>)],
) -> bool {
    let current_source_index = host
        .models_mut()
        .read(state, |state| state.active_source_index)
        .ok()
        .unwrap_or(None);
    let current_visible_index = current_source_index.and_then(|source_index| {
        visible_candidates
            .iter()
            .position(|(candidate_source, _)| *candidate_source == source_index)
    });
    let disabled = vec![false; visible_candidates.len()];
    let Some(next_visible_index) = crate::headless::cmdk_selection::next_active_index(
        &disabled,
        current_visible_index,
        forward,
        true,
    ) else {
        return false;
    };
    let next_source_index = visible_candidates[next_visible_index].0;
    let _ = host.update_model(state, |state| {
        state.active_source_index = Some(next_source_index);
        state.active_element = None;
        state.picked = None;
    });
    host.request_redraw(action_cx.window);
    true
}
