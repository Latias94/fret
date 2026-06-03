use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{BeginMenuOptions, DisclosureResponse, ImUiFacade, UiWriterImUiFacadeExt};
use open::open_begin_menu_popup_if_requested;
use trigger_flow::{BeginMenuTriggerInput, run_begin_menu_trigger_flow};

use super::menu_state;

mod open;
mod trigger_flow;

pub(in crate::imui) fn begin_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::super::imui_is_disabled(cx));
    let menu_state = menu_state::capture_begin_menu_state(ui, id);

    let trigger_flow = run_begin_menu_trigger_flow(
        ui,
        id,
        &menu_state,
        BeginMenuTriggerInput {
            label,
            enabled,
            test_id: options.test_id.clone(),
            activate_shortcut: options.activate_shortcut,
            shortcut_repeat: options.shortcut_repeat,
        },
    );
    let trigger = trigger_flow.trigger;

    open_begin_menu_popup_if_requested(
        ui,
        id,
        &menu_state,
        trigger_flow.open_menu_before,
        trigger.id(),
        trigger.rect(),
    );

    let popup_opened = super::super::popup_overlay::begin_popup_menu_with_options(
        ui,
        id,
        trigger.id(),
        options.popup,
        menu_state.menubar_policy.is_some(),
        f,
    );
    menu_state::close_disabled_popup_if_opened(ui, id, &menu_state, enabled, popup_opened);

    let render_state = menu_state::record_render_state(ui, &menu_state);

    DisclosureResponse {
        trigger,
        open: render_state.popup_open_after,
        toggled: menu_state.popup_open_before != render_state.popup_open_after,
    }
}
