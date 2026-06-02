use fret_ui::{GlobalElementId, UiHost};

use crate::imui::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt, popup_overlay};

pub(super) fn begin_submenu_popup<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    enabled: bool,
    options: PopupMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) {
    let popup_opened =
        popup_overlay::begin_popup_menu_with_options(ui, id, trigger, options, false, f);
    if !enabled && popup_opened {
        ui.close_popup(id);
    }
}
