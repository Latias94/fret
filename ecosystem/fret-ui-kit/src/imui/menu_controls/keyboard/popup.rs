use std::sync::Arc;

use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::active_trigger_behavior;

use super::super::interaction::MenuItemInteraction;

mod nav;
mod shortcut;

pub(in crate::imui::menu_controls) fn install_popup_menu_keyboard<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    behavior: &active_trigger_behavior::ActiveTriggerBehavior,
    interaction: &MenuItemInteraction,
) {
    let Some(nav_items) = nav::register_popup_menu_nav_item(cx, id) else {
        return;
    };

    let shortcut = shortcut::popup_menu_shortcut(behavior, interaction);
    cx.key_on_key_down_for(
        id,
        Arc::new(move |host, acx, down| {
            shortcut::handle_popup_menu_shortcut(host, acx, &down, &shortcut)
                || nav::move_popup_menu_focus(host, acx, &down, &nav_items, id)
        }),
    );
}
