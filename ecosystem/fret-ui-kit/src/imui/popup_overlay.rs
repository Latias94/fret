use fret_ui::{GlobalElementId, UiHost};

use super::{ImUiFacade, PopupMenuOptions, PopupModalOptions, UiWriterImUiFacadeExt};

mod context_menu;
mod menu;
mod modal;
mod state;

pub(super) use context_menu::begin_popup_context_menu_with_options;
pub(in crate::imui) use menu::{ImUiMenuNavState, ImUiPopupMenuPolicyState};
pub(super) use state::{
    close_popup, drop_popup_scope, open_popup, open_popup_at, popup_open_model,
};

pub(super) fn begin_popup_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    preserve_focus_outside_while_submenu_open: bool,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    menu::begin_popup_menu_with_options(
        ui,
        id,
        trigger,
        options,
        preserve_focus_outside_while_submenu_open,
        f,
    )
}

pub(super) fn begin_popup_modal_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupModalOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    modal::begin_popup_modal_with_options(ui, id, trigger, options, f)
}
