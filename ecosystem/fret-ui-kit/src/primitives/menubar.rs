//! Radix `Menubar` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/menubar>
//!
//! In Radix, `Menubar` uses `Menu`-like content behavior but has an additional "trigger row"
//! interaction policy (roving between triggers, hover switches menus when one is open, etc.).
//! In Fret the shared menu content/submenu behavior lives in `crate::primitives::menu`; this module
//! exists as a Radix-named facade for consumers that want to align their mental model with Radix.

use fret_runtime::Model;
use fret_ui::action::OnDismissiblePointerMove;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::menu::{root as menu_root, sub as menu_sub};
use crate::{OverlayController, OverlayPresence, OverlayRequest};

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

/// A stable per-overlay root name for a menubar menu.
pub fn menubar_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// Sync root open state and ensure submenu models exist inside a menubar menu overlay root.
#[track_caller]
pub fn menubar_sync_root_open_and_ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    is_open: bool,
    cfg: menu_sub::MenuSubmenuConfig,
) -> menu_sub::MenuSubmenuModels {
    cx.with_root_name(root_name, |cx| {
        menu_root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), cfg)
    })
}

/// Build a Radix-aligned menubar overlay request (non-modal dismissible menu).
pub fn menubar_dismissible_request<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
    root_name: String,
    content_focus: Option<GlobalElementId>,
    dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
) -> OverlayRequest {
    menu_root::dismissible_menu_request(
        cx,
        id,
        trigger,
        open,
        presence,
        children,
        root_name,
        content_focus,
        dismissible_on_pointer_move,
    )
}
