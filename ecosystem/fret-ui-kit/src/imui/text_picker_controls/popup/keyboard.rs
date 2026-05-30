use fret_ui::UiHost;

use super::super::keyboard::install_picker_keyboard_handler;
use super::types::InputTextPickerPopupInput;

pub(super) fn install_popup_keyboard_handler_if_needed<H: UiHost>(
    ui: &mut super::super::super::ImUiFacade<'_, '_, H>,
    input: &InputTextPickerPopupInput<'_>,
) {
    if !input.install_keyboard_handler {
        return;
    }
    let Some(keyboard_state) = input.keyboard_state.clone() else {
        return;
    };

    let cx = ui.cx_mut();
    let key_owner = cx.root_id();
    install_picker_keyboard_handler(
        cx,
        key_owner,
        input.model.clone(),
        input.popup_open.clone(),
        keyboard_state,
        input.visible_candidates.to_vec(),
        input.keyboard_repeat,
    );
}
