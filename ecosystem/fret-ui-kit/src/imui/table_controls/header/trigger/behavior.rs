use fret_ui::action::ActivateReason;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    KEY_CLICKED, ResponseExt, active_trigger_behavior, mark_lifecycle_instant_if_inactive,
};

pub(super) fn install_header_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    element_id: GlobalElementId,
    state: PressableState,
    enabled: bool,
    activates_on_primary: bool,
    trigger: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        element_id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions {
            primary_active: activates_on_primary,
            ..Default::default()
        },
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    if enabled && activates_on_primary {
        cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
            if reason == ActivateReason::Keyboard {
                mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
            }
            host.record_transient_event(acx, KEY_CLICKED);
            host.notify(acx);
        }));
    }

    let clicked = if activates_on_primary {
        cx.take_transient_for(element_id, KEY_CLICKED)
    } else {
        let _ = cx.take_transient_for(element_id, KEY_CLICKED);
        false
    };
    active_trigger_behavior::populate_active_trigger_response(
        cx,
        element_id,
        state,
        &behavior,
        active_trigger_behavior::ActiveTriggerResponseInput {
            enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        trigger,
    );
}
