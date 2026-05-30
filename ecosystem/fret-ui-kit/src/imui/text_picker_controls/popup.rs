use fret_ui::UiHost;

use super::super::UiWriterImUiFacadeExt;

mod item;
mod keyboard;
mod types;

use item::{InputTextPickerPopupItemInput, render_text_picker_popup_item};
use keyboard::install_popup_keyboard_handler_if_needed;
pub(super) use types::{InputTextPickerPopupInput, InputTextPickerPopupResult};

pub(super) fn render_text_picker_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: InputTextPickerPopupInput<'_>,
) -> InputTextPickerPopupResult {
    let mut picked_index = input
        .pending_keyboard_pick
        .as_ref()
        .map(|pick| pick.source_index);
    let mut picked = input
        .pending_keyboard_pick
        .as_ref()
        .map(|pick| pick.value.clone());

    let opened = ui.begin_popup_menu_with_options(input.id, input.trigger, input.popup, |ui| {
        install_popup_keyboard_handler_if_needed(ui, &input);

        for (visible_index, (source_index, candidate)) in
            input.visible_candidates.iter().enumerate()
        {
            if let Some(item_pick) = render_text_picker_popup_item(
                ui,
                InputTextPickerPopupItemInput {
                    source_index: *source_index,
                    visible_index,
                    candidate: candidate.clone(),
                    selected_value: input.selected_value.as_str(),
                    active_source_index: input.active_source_index,
                    item_test_id_base: input.item_test_id_base.clone(),
                    keyboard_state: input.keyboard_state.clone(),
                    model: input.model.clone(),
                    popup_open: input.popup_open.clone(),
                },
            ) {
                picked_index = Some(item_pick.source_index);
                picked = Some(item_pick.value);
            }
        }
    });

    InputTextPickerPopupResult {
        opened,
        picked_index,
        picked,
    }
}
