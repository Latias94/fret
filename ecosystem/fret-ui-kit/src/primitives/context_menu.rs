//! Radix `ContextMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu>
//!
//! In Radix, `ContextMenu` is built on top of `Menu` with a different trigger/open policy.
//! In Fret we share the same underlying behavior via `crate::primitives::menu` and expose
//! Radix-named entry points here for reuse outside the shadcn layer.

use std::sync::Arc;

use fret_core::MouseButton;
use fret_runtime::Model;
use fret_ui::action::OnDismissiblePointerMove;
use fret_ui::action::{OnPointerDown, PointerDownCx, UiPointerActionHost};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

pub use crate::primitives::menu::*;

use crate::primitives::menu::{root as menu_root, sub as menu_sub};
use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// A Radix-aligned pointer-down policy for opening a context menu.
///
/// Mirrors the common desktop behavior:
/// - Right click opens.
/// - (macOS) Ctrl + left click opens.
///
/// Usage (typical):
/// - wrap your trigger in a `PointerRegion`,
/// - call `cx.pointer_region_on_pointer_down(context_menu_pointer_down_policy(open.clone()))`,
/// - read `PointerRegionState::last_down` to anchor the popup at the click position.
pub fn context_menu_pointer_down_policy(open: Model<bool>) -> OnPointerDown {
    Arc::new(
        move |host: &mut dyn UiPointerActionHost, _cx: fret_ui::action::ActionCx, down: PointerDownCx| {
            let is_right_click = down.button == MouseButton::Right;
            let is_macos_ctrl_click =
                cfg!(target_os = "macos") && down.button == MouseButton::Left && down.modifiers.ctrl;

            if !is_right_click && !is_macos_ctrl_click {
                return false;
            }

            let _ = host.models_mut().update(&open, |v| *v = true);
            true
        },
    )
}

/// Wire Radix-aligned keyboard open affordances onto a context-menu trigger.
///
/// This mirrors the common desktop behavior where Shift+F10 opens the context menu.
pub fn wire_context_menu_open_on_shift_f10<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    trigger_id: GlobalElementId,
    open: Model<bool>,
) {
    crate::primitives::menu::trigger::wire_open_on_shift_f10(cx, trigger_id, open);
}

/// A stable per-overlay root name for a context menu.
pub fn context_menu_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// Sync root open state and ensure submenu models exist inside a context-menu overlay root.
#[track_caller]
pub fn context_menu_sync_root_open_and_ensure_submenu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    root_name: &str,
    is_open: bool,
    cfg: menu_sub::MenuSubmenuConfig,
) -> menu_sub::MenuSubmenuModels {
    cx.with_root_name(root_name, |cx| {
        menu_root::sync_root_open_and_ensure_submenu(cx, is_open, cx.root_id(), cfg)
    })
}

/// Build a Radix-aligned context-menu overlay request (non-modal dismissible menu).
pub fn context_menu_dismissible_request<H: UiHost>(
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
