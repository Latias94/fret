use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::UiHost;

use super::super::{PopupMenuOptions, UiWriterImUiFacadeExt};
use super::keyboard::{
    InputTextPickerKeyboardPick, InputTextPickerKeyboardState, install_picker_keyboard_handler,
};

mod item;

use item::{InputTextPickerPopupItemInput, render_text_picker_popup_item};

pub(super) struct InputTextPickerPopupInput<'a> {
    pub(super) id: &'a str,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) popup: PopupMenuOptions,
    pub(super) model: Model<String>,
    pub(super) popup_open: Model<bool>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) visible_candidates: &'a [(usize, Arc<str>)],
    pub(super) selected_value: String,
    pub(super) active_source_index: Option<usize>,
    pub(super) pending_keyboard_pick: Option<InputTextPickerKeyboardPick>,
    pub(super) item_test_id_base: Option<Arc<str>>,
    pub(super) install_keyboard_handler: bool,
    pub(super) keyboard_repeat: bool,
}

pub(super) struct InputTextPickerPopupResult {
    pub(super) opened: bool,
    pub(super) picked_index: Option<usize>,
    pub(super) picked: Option<Arc<str>>,
}

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

fn install_popup_keyboard_handler_if_needed<H: UiHost>(
    ui: &mut super::super::ImUiFacade<'_, '_, H>,
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
