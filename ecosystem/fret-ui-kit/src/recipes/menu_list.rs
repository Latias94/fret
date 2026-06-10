//! Menu-list recipe helpers built on top of shared theme resolution.
//!
//! This module owns menu row chrome defaults so overlay/menu widgets can share
//! sizing and color fallback policy without duplicating token lookups.

mod chrome;

pub use chrome::{MenuListRowChrome, resolve_menu_list_row_chrome};
