mod capture;
mod open_policy;

pub(super) use capture::{BeginMenuState, capture_begin_menu_state, record_render_state};
pub(super) use open_policy::{
    activate_menubar_trigger_if_requested, close_disabled_popup_if_opened,
    reconcile_menubar_after_trigger, resolve_open_requested, sync_open_menu_for_active_trigger,
    toggle_menu_on_trigger_click,
};
