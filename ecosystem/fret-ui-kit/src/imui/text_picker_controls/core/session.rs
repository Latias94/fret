use std::sync::Arc;

use fret_ui::UiHost;

mod candidates;
mod popup;
mod state;

use super::super::super::{InputTextPickerOptions, UiWriterImUiFacadeExt};
use super::super::open_policy::TextPickerPopupSnapshot;
use super::keyboard_state::{PreparedTextPickerKeyboard, prepare_text_picker_keyboard};

pub(super) struct PreparedTextPickerSession {
    pub(super) current: String,
    pub(super) visible_candidates: Vec<(usize, Arc<str>)>,
    pub(super) hide_for_exact_match: bool,
    pub(super) popup_open: fret_runtime::Model<bool>,
    pub(super) picker_candidate_visible: bool,
    pub(super) keyboard: PreparedTextPickerKeyboard,
    pub(super) popup_snapshot: TextPickerPopupSnapshot,
    pub(super) picker_expanded: bool,
}

pub(super) fn prepare_text_picker_session<H, W>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    candidate_sources: &[Arc<str>],
    options: &InputTextPickerOptions,
) -> PreparedTextPickerSession
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let state = state::read_text_picker_session_state(ui, id, model, options);
    let candidate_state = candidates::prepare_text_picker_session_candidates(
        &state.current,
        candidate_sources,
        options,
    );
    let keyboard = prepare_text_picker_keyboard(
        ui,
        id,
        options.keyboard_navigation,
        state.input_enabled_by_scope,
        &candidate_state.visible_candidates,
        candidate_state.hide_for_exact_match,
    );
    let popup = popup::prepare_text_picker_session_popup(
        ui,
        id,
        &state.popup_open,
        state.input_enabled_by_scope,
        candidate_state.picker_candidate_visible,
        candidate_state.hide_for_exact_match,
    );

    PreparedTextPickerSession {
        current: state.current,
        visible_candidates: candidate_state.visible_candidates,
        hide_for_exact_match: candidate_state.hide_for_exact_match,
        popup_open: state.popup_open,
        picker_candidate_visible: candidate_state.picker_candidate_visible,
        keyboard,
        popup_snapshot: popup.popup_snapshot,
        picker_expanded: popup.picker_expanded,
    }
}
