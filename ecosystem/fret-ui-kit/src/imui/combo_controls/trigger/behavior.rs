use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::KeyChord;
use fret_ui::action::ActivateReason;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::{ResponseExt, item_behavior};

pub(super) struct ComboTriggerBehaviorInput {
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_combo_trigger_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    input: ComboTriggerBehaviorInput,
    response: &mut ResponseExt,
) {
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    cx.pressable_on_activate(crate::on_activate(move |host, acx, reason| {
        if reason == ActivateReason::Keyboard {
            crate::imui::mark_lifecycle_instant_if_inactive(
                host,
                acx,
                &lifecycle_model_for_activate,
                false,
            );
        }
        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
        host.notify(acx);
    }));

    if input.enabled {
        let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
        cx.key_on_key_down_for(
            id,
            Arc::new(move |host, acx, down| {
                if let Some(shortcut) = input.activate_shortcut {
                    let matches_shortcut =
                        down.key == shortcut.key && down.modifiers == shortcut.mods;
                    if matches_shortcut
                        && (!down.repeat || input.shortcut_repeat)
                        && !down.ime_composing
                    {
                        crate::imui::mark_lifecycle_instant_if_inactive(
                            host,
                            acx,
                            &lifecycle_model_for_shortcut,
                            false,
                        );
                        host.record_transient_event(acx, crate::imui::KEY_CLICKED);
                        host.notify(acx);
                        return true;
                    }
                }

                let is_menu_key = down.key == KeyCode::ContextMenu;
                let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                if !(is_menu_key || is_shift_f10) {
                    return false;
                }

                host.record_transient_event(acx, crate::imui::KEY_CONTEXT_MENU_REQUESTED);
                host.notify(acx);
                true
            }),
        );
    }

    let clicked = cx.take_transient_for(id, crate::imui::KEY_CLICKED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        &behavior,
        item_behavior::PressableItemResponseInput {
            enabled: input.enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
