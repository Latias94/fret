use fret_ui::UiHost;

use super::super::super::super::{InputTextPickerOptions, UiWriterImUiFacadeExt};

pub(super) struct TextPickerSessionState {
    pub(super) current: String,
    pub(super) popup_open: fret_runtime::Model<bool>,
    pub(super) input_enabled_by_scope: bool,
}

pub(super) fn read_text_picker_session_state<H, W>(
    ui: &mut W,
    id: &str,
    model: &fret_runtime::Model<String>,
    options: &InputTextPickerOptions,
) -> TextPickerSessionState
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
    let popup_open = ui.popup_open_model(id);
    let input_enabled_by_scope = ui.with_cx_mut(|cx| {
        options.input.enabled && !super::super::super::super::imui_is_disabled(cx)
    });

    TextPickerSessionState {
        current,
        popup_open,
        input_enabled_by_scope,
    }
}
