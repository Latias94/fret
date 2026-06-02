use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use super::super::super::{InputTextPickerOptions, UiWriterImUiFacadeExt};
use super::super::popup::{
    InputTextPickerPopupInput, InputTextPickerPopupResult, render_text_picker_popup,
};
use super::session::PreparedTextPickerSession;

pub(super) struct TextPickerCorePopupInput<'a> {
    pub(super) id: &'a str,
    pub(super) model: Model<String>,
    pub(super) options: &'a InputTextPickerOptions,
    pub(super) session: &'a PreparedTextPickerSession,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) input_enabled: bool,
    pub(super) input_focused: bool,
    pub(super) item_test_id_base: Option<Arc<str>>,
}

pub(super) fn render_text_picker_core_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: TextPickerCorePopupInput<'_>,
) -> InputTextPickerPopupResult {
    let session = input.session;
    render_text_picker_popup(
        ui,
        InputTextPickerPopupInput {
            id: input.id,
            trigger: input.trigger,
            popup: input.options.popup,
            model: input.model,
            popup_open: session.popup_open.clone(),
            keyboard_state: session.keyboard.state.clone(),
            visible_candidates: &session.visible_candidates,
            selected_value: session.current.clone(),
            active_source_index: session.keyboard.active_source_index,
            pending_keyboard_pick: session.keyboard.pending_keyboard_pick.clone(),
            item_test_id_base: input.item_test_id_base,
            install_keyboard_handler: input.input_enabled
                && input.options.keyboard_navigation
                && input.input_focused
                && session.picker_candidate_visible
                && !session.hide_for_exact_match,
            keyboard_repeat: input.options.keyboard_repeat,
        },
    )
}
