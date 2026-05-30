use fret_ui::UiHost;

use crate::imui::UiWriterImUiFacadeExt;

use super::super::capture::BeginMenuState;

pub(in crate::imui::menu_family_controls) fn close_disabled_popup_if_opened<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    state: &BeginMenuState,
    enabled: bool,
    popup_opened: bool,
) {
    if enabled || !popup_opened {
        return;
    }

    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&state.row_open, |value| *value = false);
    });
    ui.close_popup(id);
}
