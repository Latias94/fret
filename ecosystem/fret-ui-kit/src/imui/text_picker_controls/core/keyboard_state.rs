use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::{GlobalElementId, UiHost};

use crate::imui::UiWriterImUiFacadeExt;

use super::super::keyboard::{
    InputTextPickerKeyboardPick, InputTextPickerKeyboardState, reconcile_picker_keyboard_state,
};

pub(super) struct PreparedTextPickerKeyboard {
    pub(super) state: Option<Model<InputTextPickerKeyboardState>>,
    pub(super) active_source_index: Option<usize>,
    pub(super) pending_keyboard_pick: Option<InputTextPickerKeyboardPick>,
    pub(super) active_element: Option<GlobalElementId>,
}

pub(super) fn prepare_text_picker_keyboard<H, W>(
    ui: &mut W,
    id: &str,
    keyboard_navigation: bool,
    input_enabled_by_scope: bool,
    visible_candidates: &[(usize, Arc<str>)],
    hide_for_exact_match: bool,
) -> PreparedTextPickerKeyboard
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let state = keyboard_navigation.then(|| {
        ui.with_cx_mut(|cx| {
            cx.local_model_keyed(
                format!("fret-ui-kit.imui.input-text-picker.keyboard.{id}"),
                InputTextPickerKeyboardState::default,
            )
        })
    });
    let (active_source_index, pending_keyboard_pick, active_element) = state
        .as_ref()
        .and_then(|state| {
            ui.with_cx_mut(|cx| {
                reconcile_picker_keyboard_state(
                    cx,
                    state,
                    input_enabled_by_scope,
                    visible_candidates,
                    hide_for_exact_match,
                )
            })
        })
        .map(|snapshot| {
            (
                snapshot.active_source_index,
                snapshot.pending_pick,
                snapshot.active_element,
            )
        })
        .unwrap_or((None, None, None));

    PreparedTextPickerKeyboard {
        state,
        active_source_index,
        pending_keyboard_pick,
        active_element,
    }
}
