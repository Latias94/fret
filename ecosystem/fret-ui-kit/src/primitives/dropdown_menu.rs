//! Radix `DropdownMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/dropdown-menu>
//!
//! In Radix, `DropdownMenu` is built on top of `Menu` with a trigger button and popper-based
//! placement. In Fret we share the same underlying behavior via `crate::primitives::menu` and
//! expose Radix-named entry points here for reuse outside the shadcn layer.

use fret_core::{Px, Rect};

use crate::primitives::popper;

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as dropdown_menu_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as dropdown_menu_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as dropdown_menu_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as dropdown_menu_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_arrow_keys as wire_dropdown_menu_open_on_arrow_keys;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropdownMenuPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn dropdown_menu_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "dropdown menu popper vars" (`--radix-dropdown-menu-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-dropdown-menu-content-available-width`
/// - `--radix-dropdown-menu-content-available-height`
/// - `--radix-dropdown-menu-trigger-width`
/// - `--radix-dropdown-menu-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn dropdown_menu_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> DropdownMenuPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    DropdownMenuPopperVars {
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
    use fret_core::Size;

    #[test]
    fn dropdown_menu_popper_vars_available_height_tracks_flipped_side_space() {
        let outer = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(100.0), Px(100.0)),
        );
        let anchor = Rect::new(
            Point::new(Px(10.0), Px(70.0)),
            Size::new(Px(30.0), Px(10.0)),
        );

        let placement = popper::PopperContentPlacement::new(
            popper::LayoutDirection::Ltr,
            popper::Side::Bottom,
            popper::Align::Start,
            Px(0.0),
        );
        let vars = dropdown_menu_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
    }
}
