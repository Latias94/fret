use std::sync::Arc;

use fret_ui::UiHost;

use crate::imui::{
    BeginSubmenuOptions, DisclosureResponse, ImUiFacade, UiWriterImUiFacadeExt, popup_overlay,
};

mod open_policy;
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
    let popup_open = ui.popup_open_model(id);
    let popup_policy = ui.with_cx_mut(|cx| {
        cx.provided::<popup_overlay::ImUiPopupMenuPolicyState>()
            .cloned()
    });
    let was_open_model =
        ui.with_cx_mut(|cx| cx.local_model_keyed(format!("was_open.{id}"), || false));
    let open_before = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    let was_open_before_render = ui.with_cx_mut(|cx| {
        cx.read_model(
            &was_open_model,
            fret_ui::Invalidation::Paint,
            |_app, value| *value,
        )
        .unwrap_or(false)
    });
    let submenu_value = Arc::<str>::from(id);

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        trigger::submenu_trigger(
            ui,
            label.clone(),
            trigger::SubmenuTriggerInput {
                enabled,
                open_before,
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
            open_before,
            was_open_before_render,
            submenu_value: submenu_value.clone(),
            popup_policy: popup_policy.as_ref(),
            trigger: &trigger,
        },
    );

    let popup_opened =
        popup_overlay::begin_popup_menu_with_options(ui, id, trigger.id(), options.popup, false, f);
    if !enabled && popup_opened {
        ui.close_popup(id);
    }

    let open_after = ui.with_cx_mut(|cx| {
        cx.read_model(&popup_open, fret_ui::Invalidation::Paint, |_app, value| {
            *value
        })
        .unwrap_or(false)
    });
    ui.with_cx_mut(|cx| {
        let _ = cx
            .app
            .models_mut()
            .update(&was_open_model, |value| *value = open_after);
    });

    DisclosureResponse {
        trigger,
        open: open_after,
        toggled: open_before != open_after,
    }
}
