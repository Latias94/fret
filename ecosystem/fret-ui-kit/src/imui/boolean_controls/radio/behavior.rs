use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::KeyChord;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{
    KEY_CLICKED, KEY_CONTEXT_MENU_REQUESTED, ResponseExt, item_behavior,
    mark_lifecycle_instant_if_inactive,
};

pub(super) struct RadioBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_radio_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    options: RadioBehaviorOptions,
    response: &mut ResponseExt,
) {
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
        mark_lifecycle_instant_if_inactive(host, acx, &lifecycle_model_for_activate, false);
        host.record_transient_event(acx, KEY_CLICKED);
        host.notify(acx);
    }));

    if options.enabled {
        let lifecycle_model_for_shortcut = behavior.lifecycle_model.clone();
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

                let is_menu_key = down.key == KeyCode::ContextMenu;
                let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
                if !(is_menu_key || is_shift_f10) {
                    return false;
                }

                host.record_transient_event(acx, KEY_CONTEXT_MENU_REQUESTED);
                host.notify(acx);
                true
            }),
        );
    }

    let clicked = cx.take_transient_for(id, KEY_CLICKED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        &behavior,
        item_behavior::PressableItemResponseInput {
            enabled: options.enabled,
            clicked,
            changed: false,
            lifecycle_edited: false,
        },
        response,
    );
}
