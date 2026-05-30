use std::sync::Arc;

use fret_core::KeyCode;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::primitives::menubar::trigger_row as menubar_trigger_row;

use super::MenuTriggerBehaviorInput;

pub(super) fn install_menubar_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: &MenuTriggerBehaviorInput,
) {
    let Some(menubar_policy) = input.menubar_policy.as_ref() else {
        return;
    };

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
