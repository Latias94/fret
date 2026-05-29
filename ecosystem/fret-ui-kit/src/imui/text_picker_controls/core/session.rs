use std::sync::Arc;

use fret_ui::UiHost;

use super::super::super::{InputTextPickerOptions, UiWriterImUiFacadeExt};
use super::super::candidates::resolve_text_picker_candidates;
use super::super::open_policy::{
    TextPickerPopupSnapshot, read_text_picker_popup_snapshot, text_picker_expanded,
};
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
    candidates: &[Arc<str>],
    options: &InputTextPickerOptions,
) -> PreparedTextPickerSession
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let current = ui.with_cx_mut(|cx| {
        cx.read_model(model, fret_ui::Invalidation::Paint, |_app, value| {
            value.clone()
        })
        .unwrap_or_default()
    });

    let candidate_visibility = resolve_text_picker_candidates(&current, candidates, options);
    let visible_candidates = candidate_visibility.visible_candidates;
    let hide_for_exact_match = candidate_visibility.hide_for_exact_match;
    let popup_open = ui.popup_open_model(id);
    let picker_candidate_visible = candidate_visibility.picker_candidate_visible;
    let input_enabled_by_scope =
        ui.with_cx_mut(|cx| options.input.enabled && !super::super::super::imui_is_disabled(cx));
    let keyboard = prepare_text_picker_keyboard(
        ui,
        id,
        options.keyboard_navigation,
        input_enabled_by_scope,
        &visible_candidates,
        hide_for_exact_match,
    );
    let popup_snapshot = read_text_picker_popup_snapshot(ui, id, &popup_open);
    let picker_expanded = text_picker_expanded(
        popup_snapshot.is_open,
        input_enabled_by_scope,
        picker_candidate_visible,
        hide_for_exact_match,
    );

    PreparedTextPickerSession {
        current,
        visible_candidates,
        hide_for_exact_match,
        popup_open,
        picker_candidate_visible,
        keyboard,
        popup_snapshot,
        picker_expanded,
    }
}
