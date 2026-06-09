use fret_ui::{GlobalElementId, UiHost};

use super::super::super::{UiWriterImUiFacadeExt, with_popup_store_for_id};

pub(in crate::imui::text_picker_controls) struct TextPickerPopupSnapshot {
    pub(in crate::imui::text_picker_controls) is_open: bool,
    pub(in crate::imui::text_picker_controls) panel_id: Option<GlobalElementId>,
}

pub(in crate::imui::text_picker_controls) fn read_text_picker_popup_snapshot<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
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
