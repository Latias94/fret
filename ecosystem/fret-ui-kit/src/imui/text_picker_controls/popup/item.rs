use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::UiHost;

mod row;
mod selection;

use super::super::keyboard::{InputTextPickerKeyboardPick, InputTextPickerKeyboardState};
use crate::imui::ImUiFacade;

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
    let row = row::render_text_picker_popup_item_row(
        ui,
        row::TextPickerPopupItemRowInput {
            source_index: input.source_index,
            visible_index: input.visible_index,
            candidate: input.candidate.clone(),
            selected_value: input.selected_value,
            active_source_index: input.active_source_index,
            item_test_id_base: input.item_test_id_base.clone(),
            keyboard_state: input.keyboard_state.clone(),
        },
    );

    if !row.clicked {
        return None;
    }

    selection::commit_text_picker_popup_item_selection(ui, input, row.element)
}
