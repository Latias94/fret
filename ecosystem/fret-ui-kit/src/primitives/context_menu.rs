//! Radix `ContextMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu>
//!
//! In Radix, `ContextMenu` is built on top of `Menu` with a different trigger/open policy.
//! In Fret we share the same underlying behavior via `crate::primitives::menu` and expose
//! Radix-named entry points here for reuse outside the shadcn layer.

use std::sync::Arc;

use fret_core::{MouseButton, Px, Rect, Size};
use fret_runtime::Model;
use fret_ui::action::{OnPointerDown, PointerDownCx, UiPointerActionHost};

use crate::primitives::popper;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContextMenuPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn context_menu_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    Px(anchor.size.width.0.max(min_width.0).min(outer.size.width.0))
}

/// Compute Radix-like "context menu popper vars" (`--radix-context-menu-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-context-menu-content-available-width`
/// - `--radix-context-menu-content-available-height`
/// - `--radix-context-menu-trigger-width`
/// - `--radix-context-menu-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn context_menu_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> ContextMenuPopperVars {
    let desired_w = context_menu_popper_desired_width(outer, anchor, min_width);
    let probe_desired = Size::new(desired_w, outer.size.height);
    let layout = popper::popper_content_layout_sized(outer, anchor, probe_desired, placement);
    let metrics = popper::popper_available_metrics(outer, anchor, &layout, placement.direction);

    ContextMenuPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Point;

    #[test]
    fn context_menu_popper_vars_available_height_tracks_flipped_side_space() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(Point::new(Px(10.0), Px(70.0)), Size::new(Px(1.0), Px(1.0)));

        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            popper::Side::Bottom,
            popper::Align::Start,
            Px(0.0),
        );
        let vars = context_menu_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 90.0);
    }
}
