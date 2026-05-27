use std::sync::Arc;

use fret_runtime::{KeyChord, Model};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{
    KEY_CHANGED, KEY_CLICKED, ResponseExt, active_trigger_behavior, mark_lifecycle_edit,
};

pub(super) struct SwitchBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_switch_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    model: Model<bool>,
    options: SwitchBehaviorOptions,
    response: &mut ResponseExt,
) {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions {
            primary_active: true,
            request_focus_on_press: false,
            clear_pointer_move: true,
        },
    );
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();
    let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();

    let model_for_activate = model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
        let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
        mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
        host.record_transient_event(acx, KEY_CLICKED);
        host.record_transient_event(acx, KEY_CHANGED);
        host.notify(acx);
    }));

    if options.enabled && options.focusable {
        let model_for_shortcut = model.clone();
        cx.key_on_key_down_for(
            id,
            Arc::new(move |host, acx, down| {
                if let Some(shortcut) = options.activate_shortcut {
                    let matches_shortcut =
                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                    if matches_shortcut
                        && (!down.repeat || options.shortcut_repeat)
                        && !down.ime_composing
                    {
                        let _ = host.update_model(&model_for_shortcut, |v: &mut bool| *v = !*v);
                        mark_lifecycle_edit(host, acx, &lifecycle_model_for_shortcut);
                        host.record_transient_event(acx, KEY_CLICKED);
                        host.record_transient_event(acx, KEY_CHANGED);
                        host.notify(acx);
                        return true;
                    }
                }

                false
            }),
        );
    }

    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    let changed = cx.take_transient_for(id, KEY_CHANGED);
    active_trigger_behavior::populate_active_trigger_response(
        cx,
        id,
        state,
        &behavior,
        active_trigger_behavior::ActiveTriggerResponseInput {
            enabled: options.enabled,
            clicked,
            changed,
            lifecycle_edited: changed,
        },
        response,
    );
}
