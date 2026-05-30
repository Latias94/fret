use std::sync::Arc;

use fret_core::KeyCode;
use fret_ui::{ElementContext, GlobalElementId, UiHost};

use crate::imui::KEY_CONTEXT_MENU_REQUESTED;

pub(super) fn install_context_menu_key_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
) {
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
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
