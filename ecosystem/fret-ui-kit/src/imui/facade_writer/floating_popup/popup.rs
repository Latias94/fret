mod begin;
mod state;

pub(in crate::imui::facade_writer) use begin::{
    begin_popup_context_menu_with_options, begin_popup_menu_with_options,
    begin_popup_modal_with_options,
};
pub(in crate::imui::facade_writer) use state::{
    close_popup, drop_popup_scope, open_popup, open_popup_at, popup_open_model,
};
