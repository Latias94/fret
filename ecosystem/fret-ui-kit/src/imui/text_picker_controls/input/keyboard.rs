use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::keyboard::{InputTextPickerKeyboardState, install_picker_keyboard_handler};
use crate::imui::ResponseExt;

pub(super) struct InputRootKeyboardHandlerRequest<'a> {
    pub(super) model: Model<String>,
    pub(super) popup_open: Model<bool>,
    pub(super) keyboard_state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) visible_candidates: &'a [(usize, Arc<str>)],
    pub(super) keyboard_navigation: bool,
    pub(super) keyboard_repeat: bool,
    pub(super) picker_candidate_visible: bool,
    pub(super) hide_for_exact_match: bool,
}

pub(in crate::imui::text_picker_controls) fn install_input_root_keyboard_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_id: GlobalElementId,
    response: &ResponseExt,
    request: InputRootKeyboardHandlerRequest<'_>,
) {
    if response.enabled()
        && request.keyboard_navigation
        && response.focused()
        && request.picker_candidate_visible
        && !request.hide_for_exact_match
        && let Some(state) = request.keyboard_state.clone()
    {
        install_picker_keyboard_handler(
            cx,
            root_id,
            request.model,
            request.popup_open,
            state,
            request.visible_candidates.to_vec(),
            request.keyboard_repeat,
        );
    }
}
