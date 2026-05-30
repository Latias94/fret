use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

use crate::imui::{ImUiFacade, SelectableOptions};

use super::super::keyboard::{InputTextPickerKeyboardPick, InputTextPickerKeyboardState};

pub(super) struct InputTextPickerPopupItemInput<'a> {
    pub(super) source_index: usize,
    pub(super) visible_index: usize,
    pub(super) candidate: Arc<str>,
    pub(super) selected_value: &'a str,
    pub(super) active_source_index: Option<usize>,
    pub(super) item_test_id_base: Option<Arc<str>>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) model: Model<String>,
    pub(super) popup_open: Model<bool>,
}

pub(super) fn render_text_picker_popup_item<H: UiHost>(
    ui: &mut ImUiFacade<'_, '_, H>,
    input: InputTextPickerPopupItemInput<'_>,
) -> Option<InputTextPickerKeyboardPick> {
    let checked = input.selected_value == input.candidate.as_ref();
    let active = input.active_source_index == Some(input.source_index);
    let item_test_id = input
        .item_test_id_base
        .as_ref()
        .map(|base| Arc::from(format!("{base}.option.{}", input.visible_index)));
    let response = ui.selectable_with_options(
        input.candidate.clone(),
        SelectableOptions {
            selected: checked,
            highlighted: active,
            test_id: item_test_id,
            ..Default::default()
        },
    );

    if active && let (Some(state), Some(element)) = (input.keyboard_state.as_ref(), response.id()) {
        let _ = ui
            .cx_mut()
            .app
            .models_mut()
            .update(state, |state| state.active_element = Some(element));
    }

    if !response.clicked() {
        return None;
    }

    let next_value = input.candidate.to_string();
    let _ = ui
        .cx_mut()
        .app
        .models_mut()
        .update(&input.model, |value| *value = next_value.clone());
    let _ = ui
        .cx_mut()
        .app
        .models_mut()
        .update(&input.popup_open, |open| *open = false);
    if let Some(state) = input.keyboard_state.as_ref() {
        let _ = ui.cx_mut().app.models_mut().update(state, |state| {
            state.active_source_index = Some(input.source_index);
            state.active_element = response.id();
        });
    }

    Some(InputTextPickerKeyboardPick {
        source_index: input.source_index,
        value: input.candidate,
    })
}
