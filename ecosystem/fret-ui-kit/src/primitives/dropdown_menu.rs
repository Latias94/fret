//! Radix `DropdownMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu>
//!
//! In Radix, `DropdownMenu` is built on top of `Menu` with a trigger button and popper-based
//! placement. In Fret we share the same underlying behavior via `crate::primitives::menu` and
//! expose Radix-named entry points here for reuse outside the shadcn layer.

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as dropdown_menu_dismissible_request;
pub use crate::primitives::menu::root::menu_overlay_root_name as dropdown_menu_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as dropdown_menu_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_arrow_keys as wire_dropdown_menu_open_on_arrow_keys;
