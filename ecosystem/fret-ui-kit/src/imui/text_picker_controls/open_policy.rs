use fret_core::Rect;
use fret_ui::{GlobalElementId, UiHost};

use super::super::{UiWriterImUiFacadeExt, with_popup_store_for_id};

pub(super) struct TextPickerPopupSnapshot {
    pub(super) is_open: bool,
    pub(super) panel_id: Option<GlobalElementId>,
}

pub(super) struct TextPickerOpenPolicyInput {
    pub(super) enabled: bool,
    pub(super) visible_candidates_empty: bool,
    pub(super) hide_for_exact_match: bool,
    pub(super) open_on_focus: bool,
    pub(super) input_focused: bool,
    pub(super) picker_candidate_visible: bool,
    pub(super) anchor: Option<Rect>,
}

pub(super) fn read_text_picker_popup_snapshot<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    popup_open: &fret_runtime::Model<bool>,
) -> TextPickerPopupSnapshot {
    ui.with_cx_mut(|cx| {
        let is_open = cx
            .read_model(popup_open, fret_ui::Invalidation::Paint, |_app, value| {
                *value
            })
            .unwrap_or(false);
        let panel_id = with_popup_store_for_id(cx, id, |st, _app| st.panel_id);
        TextPickerPopupSnapshot { is_open, panel_id }
    })
}

pub(super) fn text_picker_expanded(
    popup_is_open: bool,
    input_enabled_by_scope: bool,
    picker_candidate_visible: bool,
    hide_for_exact_match: bool,
) -> bool {
    popup_is_open && input_enabled_by_scope && picker_candidate_visible && !hide_for_exact_match
}

pub(super) fn apply_text_picker_open_policy<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    input: TextPickerOpenPolicyInput,
) {
    if input.enabled && (input.visible_candidates_empty || input.hide_for_exact_match) {
        ui.close_popup(id);
    }
    if input.enabled
        && input.open_on_focus
        && input.input_focused
        && input.picker_candidate_visible
        && !input.hide_for_exact_match
        && let Some(anchor) = input.anchor
    {
        ui.open_popup_at(id, anchor);
    }
}
