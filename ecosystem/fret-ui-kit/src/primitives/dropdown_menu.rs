//! Radix `DropdownMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu>
//!
//! In Radix, `DropdownMenu` is built on top of `Menu` with a trigger button and popper-based
//! placement. In Fret we share the same underlying behavior via `crate::primitives::menu` and
//! expose Radix-named entry points here for reuse outside the shadcn layer.

use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

pub use crate::primitives::menu::*;

/// Wire Radix-aligned keyboard open affordances onto a dropdown-menu trigger.
///
/// This mirrors the common Radix / APG behavior where ArrowDown/ArrowUp opens the menu.
pub fn wire_dropdown_menu_open_on_arrow_keys<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) {
    crate::primitives::menu::trigger::wire_open_on_arrow_keys(cx, trigger_id, open);
}
