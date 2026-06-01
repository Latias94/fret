use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{BeginMenuOptions, DisclosureResponse, ImUiFacade, UiWriterImUiFacadeExt};
use open::open_begin_menu_popup_if_requested;

use super::menu_state;

mod open;

pub(in crate::imui) fn begin_menu_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginMenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !super::super::imui_is_disabled(cx));
    let menu_state = menu_state::capture_begin_menu_state(ui, id);

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        super::trigger::menu_trigger_with_options(
            ui,
            Arc::from(id),
            label.clone(),
            menu_state.open_before,
            menu_state.row_open.clone(),
            menu_state.menubar_policy.clone(),
            enabled,
            options.test_id.clone(),
            options.activate_shortcut,
            options.shortcut_repeat,
        )
    });

    let open_after_trigger = menu_state.read_row_open(ui);
    menu_state::sync_open_menu_for_active_trigger(
        ui,
        id,
        &menu_state,
        open_after_trigger,
        trigger.clicked(),
        trigger.id(),
    );

    let open_menu_before = menu_state.read_menubar_open_menu(ui);
    menu_state::reconcile_menubar_after_trigger(
        ui,
        id,
        &menu_state,
        open_after_trigger,
        trigger.id(),
    );

    if enabled && trigger.clicked() {
        menu_state::toggle_menu_on_trigger_click(ui, id, &menu_state);
    }

    open_begin_menu_popup_if_requested(
        ui,
        id,
        &menu_state,
        open_menu_before,
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
