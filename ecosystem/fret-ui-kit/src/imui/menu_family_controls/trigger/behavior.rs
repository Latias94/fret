use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::{KeyChord, Model};
use fret_ui::action::ActivateReason;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    KEY_CLICKED, ResponseExt, active_trigger_behavior, mark_lifecycle_instant_if_inactive,
};
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

use super::super::ImUiMenubarPolicyState;

pub(super) struct MenuTriggerBehaviorInput {
    pub(super) logical_key: Arc<str>,
    pub(super) open_model: Model<bool>,
    pub(super) menubar_policy: Option<ImUiMenubarPolicyState>,
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_menu_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: MenuTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
        }
        host.record_transient_event(acx, KEY_CLICKED);
        host.notify(acx);
    }));

    if input.enabled {
        let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
        let activate_shortcut = input.activate_shortcut;
        let shortcut_repeat = input.shortcut_repeat;
        cx.key_on_key_down_for(
            id,
            Arc::new(move |host, acx, down| {
                if let Some(shortcut) = activate_shortcut {
                    let matches_shortcut =
                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                    if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing
                    {
                        mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_shortcut,
                            false,
                        );
                        host.record_transient_event(acx, KEY_CLICKED);
                        host.notify(acx);
                        return true;
                    }
                }

                false
            }),
        );
    }

    if let Some(menubar_policy) = input.menubar_policy.as_ref() {
        let (patient_click_sticky, patient_click_timer) =
            menubar_trigger_row::ensure_trigger_patient_click_models(cx, id);
        menubar_trigger_row::register_trigger_in_registry(
            cx,
            menubar_policy.registry.clone(),
            input.logical_key.clone(),
            id,
            input.open_model.clone(),
            input.enabled,
            None,
        );
        menubar_trigger_row::sync_trigger_row_state(
            cx,
            menubar_policy.group_active.clone(),
            id,
            input.open_model.clone(),
            patient_click_sticky.clone(),
            patient_click_timer.clone(),
            input.enabled,
            state.hovered || state.hovered_raw || state.hovered_raw_below_barrier,
            state.pressed,
            state.focused,
        );
        cx.pressable_add_on_activate(menubar_trigger_row::toggle_on_activate(
            menubar_policy.group_active.clone(),
            id,
            input.open_model.clone(),
            patient_click_sticky,
            patient_click_timer,
        ));
        let open_model_for_arrows = input.open_model.clone();
        cx.key_add_on_key_down_for(
            id,
            Arc::new(move |host, _acx, down| {
                if down.repeat {
                    return false;
                }
                match down.key {
                    KeyCode::ArrowDown | KeyCode::ArrowUp => {
                        let _ = host
                            .models_mut()
                            .update(&open_model_for_arrows, |value| *value = true);
                        true
                    }
                    _ => false,
                }
            }),
        );
    }

    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    active_trigger_behavior::populate_active_trigger_response(
        cx,
        id,
        state,
        &behavior,
        active_trigger_behavior::ActiveTriggerResponseInput {
            enabled: input.enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
