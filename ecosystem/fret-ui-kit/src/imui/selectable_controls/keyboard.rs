use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::GlobalElementId;
use fret_ui::action::{KeyDownCx, UiActionHostExt as _};
use fret_ui::{ElementContext, UiHost};

use super::super::interaction_runtime::ImUiLifecycleSessionState;

mod options;
mod popup_nav;

pub(super) use options::SelectableKeyboardOptions;

pub(super) fn install_selectable_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    focusable: bool,
    lifecycle_model: Model<ImUiLifecycleSessionState>,
    options: SelectableKeyboardOptions,
) {
    let nav_items = popup_nav::selectable_menu_nav_items(cx, id, focusable);
    let close_popup_for_key = options.close_popup;
    let lifecycle_model_for_shortcut = lifecycle_model;
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            if let Some(shortcut) = options.activate_shortcut {
                let matches_shortcut = down.key == shortcut.key && down.modifiers == shortcut.mods;
                if matches_shortcut
                    && (!down.repeat || options.shortcut_repeat)
                    && !down.ime_composing
                {
                    super::super::mark_lifecycle_instant_if_inactive(
                        host,
                        acx,
                        &lifecycle_model_for_shortcut,
                        false,
                    );
                    if let Some(open) = close_popup_for_key.as_ref() {
                        let _ = host.update_model(open, |v| *v = false);
                    }
                    host.record_transient_event(acx, super::super::KEY_CLICKED);
                    host.notify(acx);
                    return true;
                }
            }

            if record_context_menu_request(host, acx, &down) {
                return true;
            }

            let Some(nav_items) = nav_items.as_ref() else {
                return false;
            };
            popup_nav::move_popup_menu_focus(host, acx, &down, nav_items, id)
        }),
    );
}

fn record_context_menu_request(
    host: &mut dyn fret_ui::action::UiFocusActionHost,
    acx: fret_ui::action::ActionCx,
    down: &KeyDownCx,
) -> bool {
    let is_menu_key = down.key == KeyCode::ContextMenu;
    let is_shift_f10 = down.key == KeyCode::F10 && down.modifiers.shift;
    if !(is_menu_key || is_shift_f10) {
        return false;
    }

    host.record_transient_event(acx, super::super::KEY_CONTEXT_MENU_REQUESTED);
    host.notify(acx);
    true
}
