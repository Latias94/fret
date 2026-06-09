use fret_ui::UiHost;

use super::item::{InputTextPickerPopupItemInput, render_text_picker_popup_item};
use super::result::PreparedTextPickerPopupResult;
use super::types::InputTextPickerPopupInput;
use crate::imui::ImUiFacade;

pub(super) struct TextPickerPopupItemsInput<'a, 'b> {
    pub(super) input: &'a InputTextPickerPopupInput<'a>,
    pub(super) result: &'b mut PreparedTextPickerPopupResult,
}

pub(super) fn render_text_picker_popup_items<H: UiHost>(
    ui: &mut ImUiFacade<'_, '_, H>,
    input: TextPickerPopupItemsInput<'_, '_>,
) {
    for (visible_index, (source_index, candidate)) in
        input.input.visible_candidates.iter().enumerate()
    {
        if let Some(item_pick) = render_text_picker_popup_item(
            ui,
            InputTextPickerPopupItemInput {
                source_index: *source_index,
                visible_index,
                candidate: candidate.clone(),
                selected_value: input.input.selected_value.as_str(),
                active_source_index: input.input.active_source_index,
                item_test_id_base: input.input.item_test_id_base.clone(),
                keyboard_state: input.input.keyboard_state.clone(),
                model: input.input.model.clone(),
                popup_open: input.input.popup_open.clone(),
            },
        ) {
            input.result.merge_item_pick(item_pick);
        }
    }
}
