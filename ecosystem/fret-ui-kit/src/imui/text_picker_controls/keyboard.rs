use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_runtime::Model;
use fret_ui::UiHost;
use fret_ui::action::UiActionHostExt as _;

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

pub(super) fn install_picker_keyboard_handler<H: UiHost>(
    cx: &mut fret_ui::ElementContext<'_, H>,
    input_id: fret_ui::GlobalElementId,
    model: Model<String>,
    popup_open: Model<bool>,
    state: Model<InputTextPickerKeyboardState>,
    visible_candidates: Vec<(usize, Arc<str>)>,
    keyboard_repeat: bool,
) {
    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(move |host, action_cx, down| {
            if down.ime_composing
                || down.modifiers != Modifiers::default()
                || (down.repeat && !keyboard_repeat)
            {
                return false;
            }

            if visible_candidates.is_empty() {
                return false;
            }

            match down.key {
                KeyCode::ArrowDown | KeyCode::ArrowUp => {
                    let forward = down.key == KeyCode::ArrowDown;
                    let current_source_index = host
                        .models_mut()
                        .read(&state, |state| state.active_source_index)
                        .ok()
                        .unwrap_or(None);
                    let current_visible_index = current_source_index.and_then(|source_index| {
                        visible_candidates
                            .iter()
                            .position(|(candidate_source, _)| *candidate_source == source_index)
                    });
                    let disabled = vec![false; visible_candidates.len()];
                    let Some(next_visible_index) =
                        crate::headless::cmdk_selection::next_active_index(
                            &disabled,
                            current_visible_index,
                            forward,
                            true,
                        )
                    else {
                        return false;
                    };
                    let next_source_index = visible_candidates[next_visible_index].0;
                    let _ = host.update_model(&state, |state| {
                        state.active_source_index = Some(next_source_index);
                        state.active_element = None;
                        state.picked = None;
                    });
                    host.request_redraw(action_cx.window);
                    true
                }
                KeyCode::Enter | KeyCode::NumpadEnter => {
                    let current_source_index = host
                        .models_mut()
                        .read(&state, |state| state.active_source_index)
                        .ok()
                        .unwrap_or(None);
                    let Some(current_visible_index) =
                        current_source_index.and_then(|source_index| {
                            visible_candidates
                                .iter()
                                .position(|(candidate_source, _)| *candidate_source == source_index)
                        })
                    else {
                        return false;
                    };
                    let (source_index, candidate) =
                        visible_candidates[current_visible_index].clone();
                    let next_value = candidate.to_string();
                    let _ = host.update_model(&model, |value| *value = next_value);
                    let _ = host.update_model(&popup_open, |open| *open = false);
                    let _ = host.update_model(&state, |state| {
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
                _ => false,
            }
        }),
    );
}
