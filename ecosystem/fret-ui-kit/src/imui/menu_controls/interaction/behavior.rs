use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::active_trigger_behavior;

use super::super::keyboard;
use super::MenuItemInteraction;

mod activation;
mod response;

pub(super) use activation::dispatch_menu_item_action;
pub(super) use response::populate_menu_item_response;

pub(super) fn install_menu_item_behavior<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    interaction: &MenuItemInteraction,
) -> active_trigger_behavior::ActiveTriggerBehavior {
    let behavior = active_trigger_behavior::install_active_trigger_behavior(
        cx,
        id,
        active_trigger_behavior::ActiveTriggerBehaviorOptions::default(),
    );

    if !interaction.enabled {
        return behavior;
    }

    activation::install_menu_item_activate_handler(cx, &behavior, interaction);
    keyboard::install_popup_menu_keyboard(cx, id, &behavior, interaction);
    keyboard::install_menubar_keyboard(cx, id, interaction.menubar_policy.as_ref());
    behavior
}
