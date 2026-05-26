use std::sync::Arc;

use fret_ui::action::{DismissReason, OnCloseAutoFocus, OnDismissRequest};
use fret_ui::{GlobalElementId, UiHost};

use super::super::{ImUiFacade, PopupMenuOptions, UiWriterImUiFacadeExt};
use crate::primitives::menu::root as menu_root;
use crate::{OverlayController, OverlayPresence};

mod panel;
mod policy;

use panel::build_popup_menu;
use policy::popup_menu_policy_state_for_root;
pub(in crate::imui) use policy::{ImUiMenuNavState, ImUiPopupMenuPolicyState};

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
        let open = super::super::with_popup_store_for_id(cx, id, |st, _app| st.open.clone());
        let trigger_id = trigger.unwrap_or(overlay_id);
        let initial_focus = if options.auto_focus {
            menu_root::MenuInitialFocusTargets::new()
                .keyboard_entry_focus(built.first_item)
                .pointer_content_focus(built.content_focus)
        } else {
            menu_root::MenuInitialFocusTargets::new()
        };
        let on_dismiss_request = if preserve_focus_outside_while_submenu_open {
            let submenu_models = popup_policy.submenu_models.clone();
            let open_for_dismiss = open.clone();
            Some(Arc::new(
                move |host: &mut dyn fret_ui::action::UiActionHost,
                      _acx,
                      req: &mut fret_ui::action::DismissRequestCx| {
                    if matches!(req.reason, DismissReason::FocusOutside) {
                        let submenu_open = host
                            .models_mut()
                            .read(&submenu_models.open_value, |value| value.clone())
                            .ok()
                            .flatten();
                        if submenu_open.is_some() {
                            req.prevent_default();
                            return;
                        }
                    }
                    let _ = host
                        .models_mut()
                        .update(&open_for_dismiss, |value| *value = false);
                },
            ) as OnDismissRequest)
        } else {
            None
        };
        let on_close_auto_focus = menubar_policy.as_ref().map(|policy| {
            let suppress_close_auto_focus = policy.suppress_close_auto_focus_once.clone();
            Arc::new(
                move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                      _acx,
                      req: &mut fret_ui::action::AutoFocusRequestCx| {
                    let suppress = host
                        .models_mut()
                        .read(&suppress_close_auto_focus, |value| *value)
                        .ok()
                        .unwrap_or(false);
                    if !suppress {
                        return;
                    }
                    let _ = host
                        .models_mut()
                        .update(&suppress_close_auto_focus, |value| *value = false);
                    req.prevent_default();
                },
            ) as OnCloseAutoFocus
        });
        let req = menu_root::dismissible_menu_request_with_modal_and_dismiss_handler(
            cx,
            overlay_id,
            trigger_id,
            open,
            OverlayPresence::instant(true),
            built.children,
            root_name.clone(),
            initial_focus,
            None,
            on_close_auto_focus,
            on_dismiss_request,
            Some(menu_root::submenu_pointer_move_handler(
                popup_policy.submenu_models.clone(),
                popup_policy.submenu_cfg,
            )),
            options.modal,
        );
        OverlayController::request(cx, req);
    });

    true
}
