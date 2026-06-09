use fret_ui::{GlobalElementId, UiHost};

use super::super::super::keyboard::InputTextPickerKeyboardPick;
use super::InputTextPickerPopupItemInput;
use crate::imui::ImUiFacade;

pub(super) fn commit_text_picker_popup_item_selection<H: UiHost>(
    ui: &mut ImUiFacade<'_, '_, H>,
    input: InputTextPickerPopupItemInput<'_>,
    element: Option<GlobalElementId>,
) -> Option<InputTextPickerKeyboardPick> {
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
            state.active_element = element;
        });
    }

    Some(InputTextPickerKeyboardPick {
        source_index: input.source_index,
        value: input.candidate,
    })
}
