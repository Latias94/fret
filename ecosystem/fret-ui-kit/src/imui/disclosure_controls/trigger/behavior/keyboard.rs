use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use super::super::super::spec::DisclosureSpec;

pub(super) fn install_disclosure_trigger_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open_model: Model<bool>,
    spec: &DisclosureSpec,
) {
    let has_children = spec.has_children();
    let activate_shortcut = spec.activate_shortcut;
    let shortcut_repeat = spec.shortcut_repeat;
    cx.key_on_key_down_for(
        trigger_id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut && (!down.repeat || shortcut_repeat) && !down.ime_composing {
                    host.record_transient_event(acx, crate::imui::KEY_CLICKED);
                    if has_children {
                        let _ = host
                            .models_mut()
                            .update(&open_model, |value| *value = !*value);
                    }
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
