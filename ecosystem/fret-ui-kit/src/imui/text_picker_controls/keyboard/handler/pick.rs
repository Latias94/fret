use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiActionHost, UiActionHostExt as _};

use super::super::{InputTextPickerKeyboardPick, InputTextPickerKeyboardState};

pub(super) fn commit_picker_highlight(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    model: &Model<String>,
    popup_open: &Model<bool>,
    state: &Model<InputTextPickerKeyboardState>,
    visible_candidates: &[(usize, Arc<str>)],
) -> bool {
    let current_source_index = host
        .models_mut()
        .read(state, |state| state.active_source_index)
        .ok()
        .unwrap_or(None);
    let Some(current_visible_index) = current_source_index.and_then(|source_index| {
        visible_candidates
            .iter()
            .position(|(candidate_source, _)| *candidate_source == source_index)
    }) else {
        return false;
    };

    let (source_index, candidate) = visible_candidates[current_visible_index].clone();
    let next_value = candidate.to_string();
    let _ = host.update_model(model, |value| *value = next_value);
    let _ = host.update_model(popup_open, |open| *open = false);
    let _ = host.update_model(state, |state| {
        state.active_source_index = Some(source_index);
        state.active_element = None;
        state.picked = Some(InputTextPickerKeyboardPick {
            source_index,
            value: candidate,
        });
    });
    host.request_redraw(action_cx.window);
    true
}
