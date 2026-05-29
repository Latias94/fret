use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::panel::PopupMenuBuilt;
use super::policy::ImUiPopupMenuPolicyState;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::imui::{PopupMenuOptions, with_popup_store_for_id};
use crate::primitives::menu::root as menu_root;
use crate::{OverlayController, OverlayPresence};
use handlers::{popup_menu_on_close_auto_focus, popup_menu_on_dismiss_request};

mod handlers;

pub(super) struct PopupMenuOverlayRequestInput<'a> {
    pub(super) id: &'a str,
    pub(super) overlay_id: GlobalElementId,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) root_name: String,
    pub(super) options: PopupMenuOptions,
    pub(super) popup_policy: ImUiPopupMenuPolicyState,
    pub(super) menubar_policy: Option<ImUiMenubarPolicyState>,
    pub(super) preserve_focus_outside_while_submenu_open: bool,
    pub(super) built: PopupMenuBuilt,
}

pub(super) fn request_popup_menu_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input: PopupMenuOverlayRequestInput<'_>,
) {
    let open = with_popup_store_for_id(cx, input.id, |st, _app| st.open.clone());
    let trigger_id = input.trigger.unwrap_or(input.overlay_id);
    let built = input.built;
    let initial_focus = if input.options.auto_focus {
        menu_root::MenuInitialFocusTargets::new()
            .keyboard_entry_focus(built.first_item)
            .pointer_content_focus(built.content_focus)
    } else {
        menu_root::MenuInitialFocusTargets::new()
    };
    let on_dismiss_request = popup_menu_on_dismiss_request(
        input.preserve_focus_outside_while_submenu_open,
        input.popup_policy.clone(),
        open.clone(),
    );
    let on_close_auto_focus = popup_menu_on_close_auto_focus(input.menubar_policy.as_ref());
    let req = menu_root::dismissible_menu_request_with_modal_and_dismiss_handler(
        cx,
        input.overlay_id,
        trigger_id,
        open,
        OverlayPresence::instant(true),
        built.children,
        input.root_name,
        initial_focus,
        None,
        on_close_auto_focus,
        on_dismiss_request,
        Some(menu_root::submenu_pointer_move_handler(
            input.popup_policy.submenu_models.clone(),
            input.popup_policy.submenu_cfg,
        )),
        input.options.modal,
    );
    OverlayController::request(cx, req);
}
