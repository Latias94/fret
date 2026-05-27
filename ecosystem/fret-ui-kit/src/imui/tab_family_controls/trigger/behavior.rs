use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::action::ActivateReason;
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{
    KEY_CLICKED, ResponseExt, active_trigger_behavior, mark_lifecycle_instant_if_inactive,
};

pub(super) struct TabTriggerBehaviorInput {
    pub(super) selected_model: Model<Option<Arc<str>>>,
    pub(super) tab_id: Arc<str>,
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_tab_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: TabTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    if input.enabled {
        let selected_model_for_activate = input.selected_model.clone();
        let tab_id_for_activate = input.tab_id.clone();
        cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
            if reason == ActivateReason::Keyboard {
                mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
            }
            let _ = host.update_model(&selected_model_for_activate, |value| {
                *value = Some(tab_id_for_activate.clone())
            });
            host.record_transient_event(acx, KEY_CLICKED);
            host.notify(acx);
        }));

        install_tab_trigger_shortcut(cx, id, &behavior, &input);
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

fn install_tab_trigger_shortcut<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    input: &TabTriggerBehaviorInput,
) {
    let selected_model_for_shortcut = input.selected_model.clone();
    let tab_id_for_shortcut = input.tab_id.clone();
    let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
    let activate_shortcut = input.activate_shortcut;
    let shortcut_repeat = input.shortcut_repeat;
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing {
                    mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_shortcut,
                        false,
                    );
                    let _ = host.update_model(&selected_model_for_shortcut, |value| {
                        *value = Some(tab_id_for_shortcut.clone())
                    });
                    host.record_transient_event(acx, KEY_CLICKED);
                    host.notify(acx);
                    return true;
                }
            }

            false
        }),
    );
}
