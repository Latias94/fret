use fret_ui::UiHost;

use super::super::UiWriterImUiFacadeExt;

mod item;
mod items;
mod keyboard;
mod result;
mod types;

use items::{TextPickerPopupItemsInput, render_text_picker_popup_items};
use keyboard::install_popup_keyboard_handler_if_needed;
use result::PreparedTextPickerPopupResult;
pub(super) use types::{InputTextPickerPopupInput, InputTextPickerPopupResult};

pub(super) fn render_text_picker_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: InputTextPickerPopupInput<'_>,
) -> InputTextPickerPopupResult {
    let mut result =
        PreparedTextPickerPopupResult::from_pending(input.pending_keyboard_pick.as_ref());

    let opened = ui.begin_popup_menu_with_options(input.id, input.trigger, input.popup, |ui| {
        install_popup_keyboard_handler_if_needed(ui, &input);
        render_text_picker_popup_items(
            ui,
            TextPickerPopupItemsInput {
                input: &input,
                result: &mut result,
            },
        );
    });

    result.finish(opened)
}
