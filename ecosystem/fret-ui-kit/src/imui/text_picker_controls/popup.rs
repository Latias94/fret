use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::UiHost;

use super::super::{PopupMenuOptions, SelectableOptions, UiWriterImUiFacadeExt};
use super::keyboard::{
    InputTextPickerKeyboardPick, InputTextPickerKeyboardState, install_picker_keyboard_handler,
};

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
            let checked = input.selected_value.as_str() == candidate.as_ref();
            let active = input.active_source_index == Some(*source_index);
            let item_test_id = input
                .item_test_id_base
                .as_ref()
                .map(|base| Arc::from(format!("{base}.option.{visible_index}")));
            let response = ui.selectable_with_options(
                candidate.clone(),
                SelectableOptions {
                    selected: checked,
                    highlighted: active,
                    test_id: item_test_id,
                    ..Default::default()
                },
            );
            if active
                && let (Some(state), Some(element)) = (input.keyboard_state.as_ref(), response.id())
            {
                let _ = ui
                    .cx_mut()
                    .app
                    .models_mut()
                    .update(state, |state| state.active_element = Some(element));
            }
            if response.clicked() {
                let next_value = candidate.to_string();
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
                        state.active_source_index = Some(*source_index);
                        state.active_element = response.id();
                    });
                }
                picked_index = Some(*source_index);
                picked = Some(candidate.clone());
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
