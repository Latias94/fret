//! Internal popup-scope state storage for immediate-mode helpers.

mod drop_scope;
mod entry;
mod lifecycle;
mod state;

pub(super) use drop_scope::drop_popup_scope_for_id;
pub(super) use entry::{popup_render_generation_for_window, with_popup_store_for_id};
