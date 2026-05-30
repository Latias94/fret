mod active_trigger;
mod disabled;
mod resolve;
mod toggle;

pub(in crate::imui::menu_family_controls) use active_trigger::{
    activate_menubar_trigger_if_requested, reconcile_menubar_after_trigger,
    sync_open_menu_for_active_trigger,
};
pub(in crate::imui::menu_family_controls) use disabled::close_disabled_popup_if_opened;
pub(in crate::imui::menu_family_controls) use resolve::resolve_open_requested;
pub(in crate::imui::menu_family_controls) use toggle::toggle_menu_on_trigger_click;
