use fret_runtime::Model;
use fret_ui::UiHost;

use super::super::super::super::UiWriterImUiFacadeExt;
use super::super::super::open_policy::{
    TextPickerPopupSnapshot, read_text_picker_popup_snapshot, text_picker_expanded,
};

pub(super) struct PreparedTextPickerPopup {
    pub(super) popup_snapshot: TextPickerPopupSnapshot,
    pub(super) picker_expanded: bool,
}

pub(super) fn prepare_text_picker_session_popup<H, W>(
    ui: &mut W,
    id: &str,
    popup_open: &Model<bool>,
    input_enabled_by_scope: bool,
    picker_candidate_visible: bool,
    hide_for_exact_match: bool,
) -> PreparedTextPickerPopup
where
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
{
    let popup_snapshot = read_text_picker_popup_snapshot(ui, id, popup_open);
    let picker_expanded = text_picker_expanded(
        popup_snapshot.is_open,
        input_enabled_by_scope,
        picker_candidate_visible,
        hide_for_exact_match,
    );

    PreparedTextPickerPopup {
        popup_snapshot,
        picker_expanded,
    }
}
