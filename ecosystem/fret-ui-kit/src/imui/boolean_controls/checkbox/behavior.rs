use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::{KeyChord, Model};
use fret_ui::action::UiActionHostExt as _;
use fret_ui::element::PressableState;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::{
    KEY_CHANGED, KEY_CONTEXT_MENU_REQUESTED, ResponseExt, item_behavior, mark_lifecycle_edit,
};

pub(super) struct CheckboxBehaviorOptions {
    pub(super) enabled: bool,
    pub(super) activate_shortcut: Option<KeyChord>,
    pub(super) shortcut_repeat: bool,
}

pub(super) fn install_checkbox_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    state: PressableState,
    model: Model<bool>,
    options: CheckboxBehaviorOptions,
    response: &mut ResponseExt,
) {
    let behavior = item_behavior::install_pressable_item_behavior(cx, id);
    let lifecycle_model_for_activate = behavior.lifecycle_model.clone();

    let model_for_activate = model.clone();
    cx.pressable_on_activate(crate::on_activate(move |host, acx, _reason| {
        let _ = host.update_model(&model_for_activate, |v: &mut bool| *v = !*v);
        mark_lifecycle_edit(host, acx, &lifecycle_model_for_activate);
        host.record_transient_event(acx, KEY_CHANGED);
        host.notify(acx);
    }));

    if options.enabled {
        let model_for_shortcut = model.clone();
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
                        let _ = host.update_model(&model_for_shortcut, |v: &mut bool| *v = !*v);
                        mark_lifecycle_edit(host, acx, &lifecycle_model_for_shortcut);
                        host.record_transient_event(acx, KEY_CHANGED);
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

    let changed = cx.take_transient_for(id, KEY_CHANGED);
    item_behavior::populate_pressable_item_response(
        cx,
        id,
        state,
        &behavior,
        item_behavior::PressableItemResponseInput {
            enabled: options.enabled,
            clicked: false,
            changed,
            lifecycle_edited: changed,
        },
        response,
    );
}
