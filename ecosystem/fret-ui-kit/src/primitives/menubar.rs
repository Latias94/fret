//! Radix `Menubar` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/menubar>
//!
//! In Radix, `Menubar` uses `Menu`-like content behavior but has an additional "trigger row"
//! interaction policy (roving between triggers, hover switches menus when one is open, etc.).
//! In Fret the shared menu content/submenu behavior lives in `crate::primitives::menu`; this module
//! exists as a Radix-named facade for consumers that want to align their mental model with Radix.

pub mod trigger_row;

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as menubar_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as menubar_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as menubar_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as menubar_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_arrow_keys as wire_menubar_open_on_arrow_keys;
