use std::sync::Arc;

use fret_ui::UiHost;

use super::submenu_state;
use crate::imui::{
    BeginSubmenuOptions, DisclosureResponse, ImUiFacade, MenuItemOptions, UiWriterImUiFacadeExt,
    menu_controls, popup_overlay,
};
use crate::primitives::menu::sub_trigger;

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
    let popup_estimated_size = options.popup.estimated_size;

    let trigger = ui.push_id(format!("{id}.trigger"), |ui| {
        menu_controls::menu_item_with_options_and_pressable_hook(
            ui,
            label.clone(),
            MenuItemOptions {
                enabled,
                test_id: options.test_id.clone(),
                submenu: true,
                expanded: Some(open_before),
                activate_shortcut: options.activate_shortcut,
                shortcut_repeat: options.shortcut_repeat,
                ..Default::default()
            },
            {
                let popup_policy = popup_policy.clone();
                let submenu_value = submenu_value.clone();
                move |cx, state, item_id, item_enabled| {
                    let Some(popup_policy) = popup_policy.as_ref() else {
                        return;
                    };
                    let geometry_hint = sub_trigger::MenuSubTriggerGeometryHint {
                        outer: cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
                        desired: popup_estimated_size,
                    };
                    let _ = sub_trigger::wire(
                        cx,
                        state,
                        item_id,
                        !item_enabled,
                        true,
                        submenu_value.clone(),
                        &popup_policy.submenu_models,
                        popup_policy.submenu_cfg,
                        Some(geometry_hint),
                    );
                }
            },
        )
    });

    if let Some(policy) = popup_policy.as_ref() {
        let open_submenu_before = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.submenu_models.open_value,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        });
        let is_selected_before = open_submenu_before
            .as_ref()
            .is_some_and(|value| value.as_ref() == id);
        if enabled && trigger.clicked() {
            if open_before && is_selected_before {
                submenu_state::clear_imui_submenu(
                    ui,
                    policy,
                    submenu_value.as_ref(),
                    trigger.id(),
                    trigger.rect().is_none(),
                );
            } else if !is_selected_before {
                submenu_state::select_imui_submenu(ui, policy, submenu_value.clone(), trigger.id());
            }
        }

        if !open_before && was_open_before_render {
            submenu_state::clear_imui_submenu(
                ui,
                policy,
                submenu_value.as_ref(),
                trigger.id(),
                true,
            );
        }

        let open_submenu_now = ui.with_cx_mut(|cx| {
            cx.read_model(
                &policy.submenu_models.open_value,
                fret_ui::Invalidation::Paint,
                |_app, value| value.clone(),
            )
            .unwrap_or(None)
        });
        let should_open = open_submenu_now
            .as_ref()
            .is_some_and(|value| value.as_ref() == id);
        let should_close = open_before && !should_open;

        if should_close {
            ui.close_popup(id);
        } else if should_open && let Some(anchor) = trigger.rect() {
            ui.open_popup_at(id, anchor);
        }
    }

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
