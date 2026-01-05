//! Radix `Menubar` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/menubar>
//!
//! In Radix, `Menubar` uses `Menu`-like content behavior but has an additional "trigger row"
//! interaction policy (roving between triggers, hover switches menus when one is open, etc.).
//! In Fret the shared menu content/submenu behavior lives in `crate::primitives::menu`; this module
//! exists as a Radix-named facade for consumers that want to align their mental model with Radix.

use fret_runtime::Model;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

pub use crate::primitives::menu::*;

/// Wire Radix-aligned keyboard open affordances onto a menubar trigger.
///
/// This mirrors the common Radix / APG behavior where ArrowDown/ArrowUp opens the menu.
pub fn wire_menubar_open_on_arrow_keys<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) {
    crate::primitives::menu::trigger::wire_open_on_arrow_keys(cx, trigger_id, open);
}
