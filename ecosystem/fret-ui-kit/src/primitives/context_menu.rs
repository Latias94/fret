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
use fret_ui::action::{OnPointerDown, PointerDownCx, UiPointerActionHost};

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as context_menu_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as context_menu_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as context_menu_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as context_menu_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_shift_f10 as wire_context_menu_open_on_shift_f10;

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
        move |host: &mut dyn UiPointerActionHost,
              cx: fret_ui::action::ActionCx,
              down: PointerDownCx| {
            let is_right_click = down.button == MouseButton::Right;
            let is_macos_ctrl_click = cfg!(target_os = "macos")
                && down.button == MouseButton::Left
                && down.modifiers.ctrl;

            if !is_right_click && !is_macos_ctrl_click {
                return false;
            }

            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(cx.window);
            true
        },
    )
}
