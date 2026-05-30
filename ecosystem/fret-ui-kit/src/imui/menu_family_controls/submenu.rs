use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{
    BeginSubmenuOptions, DisclosureResponse, ImUiFacade, UiWriterImUiFacadeExt, popup_overlay,
};

mod open_policy;
mod popup;
mod state;
mod trigger;

pub(in crate::imui) fn begin_submenu_with_options<
    H: UiHost,
    W: UiWriterImUiFacadeExt<H> + ?Sized,
>(
    ui: &mut W,
    id: &str,
    label: Arc<str>,
    options: BeginSubmenuOptions,
    f: impl for<'cx2, 'a2> FnOnce(&mut ImUiFacade<'cx2, 'a2, H>),
) -> DisclosureResponse {
    let enabled = options.enabled && ui.with_cx_mut(|cx| !crate::imui::imui_is_disabled(cx));
    let popup_policy = ui.with_cx_mut(|cx| {
        cx.provided::<popup_overlay::ImUiPopupMenuPolicyState>()
            .cloned()
    });
    let open_state = state::submenu_open_snapshot(ui, id);
    let submenu_value = Arc::<str>::from(id);

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        trigger::submenu_trigger(
            ui,
            label.clone(),
            trigger::SubmenuTriggerInput {
                enabled,
                open_before: open_state.open_before,
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
                test_id: options.test_id.clone(),
                popup_estimated_size: options.popup.estimated_size,
                popup_policy: popup_policy.clone(),
                submenu_value: submenu_value.clone(),
            },
        )
    });

    open_policy::reconcile_submenu_after_trigger(
        ui,
        open_policy::SubmenuOpenPolicyInput {
            id,
            enabled,
            open_before: open_state.open_before,
            was_open_before_render: open_state.was_open_before_render,
            submenu_value: submenu_value.clone(),
            popup_policy: popup_policy.as_ref(),
            trigger: &trigger,
        },
    );

    popup::begin_submenu_popup(ui, id, trigger.id(), enabled, options.popup, f);

    let open_after = state::read_submenu_open_after(ui, &open_state.popup_open);
    state::record_submenu_open_after(ui, &open_state.was_open_model, open_after);

    DisclosureResponse {
        trigger,
        open: open_after,
        toggled: open_state.open_before != open_after,
    }
}
