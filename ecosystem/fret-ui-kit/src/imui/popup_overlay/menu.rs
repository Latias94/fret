use fret_ui::{GlobalElementId, UiHost};

use super::super::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};
use crate::OverlayController;

mod panel;
mod policy;
mod request;

use panel::build_popup_menu;
use policy::popup_menu_policy_state_for_root;
pub(in crate::imui) use policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};
use request::{PopupMenuOverlayRequestInput, request_popup_menu_overlay};

pub(super) fn begin_popup_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    trigger: Option<GlobalElementId>,
    options: PopupMenuOptions,
    preserve_focus_outside_while_submenu_open: bool,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> bool {
    let overlay_id = ui.with_cx_mut(|cx| {
        let overlay_key = format!("fret-ui-kit.imui.popup.overlay.{id}");
        cx.named(overlay_key.as_str(), |cx| cx.root_id())
    });
    let root_name = OverlayController::popover_root_name(overlay_id);
    let popup_policy = popup_menu_policy_state_for_root(ui, id, root_name.as_str());
    let menubar_policy = ui.with_cx_mut(|cx| {
        cx.provided::<super::super::menu_family_controls::ImUiMenubarPolicyState>()
            .cloned()
    });
    let Some(built) = build_popup_menu(
        ui,
        id,
        root_name.as_str(),
        options,
        popup_policy.clone(),
        menubar_policy.clone(),
        f,
    ) else {
        return false;
    };

    ui.with_cx_mut(|cx| {
        request_popup_menu_overlay(
            cx,
            PopupMenuOverlayRequestInput {
                id,
                overlay_id,
                trigger,
                root_name,
                options,
                popup_policy,
                menubar_policy,
                preserve_focus_outside_while_submenu_open,
                built,
            },
        );
    });

    true
}
