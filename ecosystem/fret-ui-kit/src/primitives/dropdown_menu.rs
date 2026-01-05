//! Radix `DropdownMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu>
//!
//! In Radix, `DropdownMenu` is built on top of `Menu` with a trigger button and popper-based
//! placement. In Fret we share the same underlying behavior via `crate::primitives::menu` and
//! expose Radix-named entry points here for reuse outside the shadcn layer.

use fret_runtime::Model;
use fret_ui::action::OnDismissiblePointerMove;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::primitives::menu::{root as menu_root, sub as menu_sub};
use crate::{OverlayController, OverlayPresence, OverlayRequest};

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

/// A stable per-overlay root name for a dropdown menu.
pub fn dropdown_menu_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// Sync root open state and ensure submenu models exist inside a dropdown-menu overlay root.
#[track_caller]
pub fn dropdown_menu_sync_root_open_and_ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    is_open: bool,
    cfg: menu_sub::MenuSubmenuConfig,
) -> menu_sub::MenuSubmenuModels {
    cx.with_root_name(root_name, |cx| {
        menu_root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), cfg)
    })
}

/// Build a Radix-aligned dropdown-menu overlay request (non-modal dismissible menu).
pub fn dropdown_menu_dismissible_request<H: UiHost>(
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
