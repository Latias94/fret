use std::sync::Arc;

use super::super::policy::ImUiPopupMenuPolicyState;
use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::primitives::dismissable_layer::{DismissReason, OnDismissRequest};
use crate::primitives::focus_scope::{AutoFocusRequestCx, OnCloseAutoFocus};

pub(super) fn popup_menu_on_dismiss_request(
    preserve_focus_outside_while_submenu_open: bool,
    popup_policy: ImUiPopupMenuPolicyState,
    open: fret_runtime::Model<bool>,
) -> Option<OnDismissRequest> {
    if !preserve_focus_outside_while_submenu_open {
        return None;
    }

    let submenu_models = popup_policy.submenu_models;
    Some(Arc::new(
        move |host: &mut dyn fret_ui::action::UiActionHost,
              _acx,
              req: &mut crate::primitives::dismissable_layer::DismissRequestCx| {
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
            let _ = host.models_mut().update(&open, |value| *value = false);
        },
    ) as OnDismissRequest)
}

pub(super) fn popup_menu_on_close_auto_focus(
    menubar_policy: Option<&ImUiMenubarPolicyState>,
) -> Option<OnCloseAutoFocus> {
    menubar_policy.map(|policy| {
        let suppress_close_auto_focus = policy.suppress_close_auto_focus_once.clone();
        Arc::new(
            move |host: &mut dyn fret_ui::action::UiFocusActionHost,
                  _acx,
                  req: &mut AutoFocusRequestCx| {
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
    })
}
