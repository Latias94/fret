use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{ResponseExt, UiWriterImUiFacadeExt, popup_overlay::ImUiPopupMenuPolicyState};

use super::super::submenu_state;

mod read;

use read::read_open_submenu;

pub(super) struct SubmenuOpenPolicyInput<'a> {
    pub(super) id: &'a str,
    pub(super) enabled: bool,
    pub(super) open_before: bool,
    pub(super) was_open_before_render: bool,
    pub(super) submenu_value: Arc<str>,
    pub(super) popup_policy: Option<&'a ImUiPopupMenuPolicyState>,
    pub(super) trigger: &'a ResponseExt,
}

pub(super) fn reconcile_submenu_after_trigger<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    input: SubmenuOpenPolicyInput<'_>,
) {
    let Some(policy) = input.popup_policy else {
        return;
    };

    let open_submenu_before = read_open_submenu(ui, policy);
    let is_selected_before = open_submenu_before
        .as_ref()
        .is_some_and(|value| value.as_ref() == input.id);
    if input.enabled && input.trigger.clicked() {
        if input.open_before && is_selected_before {
            submenu_state::clear_imui_submenu(
                ui,
                policy,
                input.submenu_value.as_ref(),
                input.trigger.id(),
                input.trigger.rect().is_none(),
            );
        } else if !is_selected_before {
            submenu_state::select_imui_submenu(
                ui,
                policy,
                input.submenu_value.clone(),
                input.trigger.id(),
            );
        }
    }

    if !input.open_before && input.was_open_before_render {
        submenu_state::clear_imui_submenu(
            ui,
            policy,
            input.submenu_value.as_ref(),
            input.trigger.id(),
            true,
        );
    }

    let open_submenu_now = read_open_submenu(ui, policy);
    let should_open = open_submenu_now
        .as_ref()
        .is_some_and(|value| value.as_ref() == input.id);
    let should_close = input.open_before && !should_open;

    if should_close {
        ui.close_popup(input.id);
    } else if should_open && let Some(anchor) = input.trigger.rect() {
        ui.open_popup_at(input.id, anchor);
    }
}
