use std::sync::Arc;

use fret_core::{KeyCode, Modifiers};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::menu_family_controls::ImUiMenubarPolicyState;
use crate::primitives::menubar::trigger_row as menubar_trigger_row;

pub(in crate::imui::menu_controls) fn install_menubar_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    menubar_policy: Option<&ImUiMenubarPolicyState>,
) {
    let Some(menubar_policy) = menubar_policy else {
        return;
    };

    let suppress_close_auto_focus = menubar_policy.suppress_close_auto_focus_once.clone();
    cx.key_prepend_on_key_down_for(
        id,
        Arc::new(move |host, _acx, down| {
            if down.repeat || down.modifiers != Modifiers::default() {
                return false;
            }
            if matches!(down.key, KeyCode::ArrowLeft | KeyCode::ArrowRight) {
                let _ = host
                    .models_mut()
                    .update(&suppress_close_auto_focus, |value| *value = true);
            }
            false
        }),
    );
    menubar_trigger_row::wire_switch_open_menu_on_horizontal_arrows(
        cx,
        id,
        menubar_policy.group_active.clone(),
        menubar_policy.registry.clone(),
    );
}
