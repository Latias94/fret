//! Radix `Menubar` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/menubar>
//!
//! In Radix, `Menubar` uses `Menu`-like content behavior but has an additional "trigger row"
//! interaction policy (roving between triggers, hover switches menus when one is open, etc.).
//! In Fret the shared menu content/submenu behavior lives in `crate::primitives::menu`; this module
//! exists as a Radix-named facade for consumers that want to align their mental model with Radix.

use fret_core::{Px, Rect};

use crate::primitives::popper;

pub mod trigger_row;

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as menubar_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as menubar_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as menubar_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as menubar_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_arrow_keys as wire_menubar_open_on_arrow_keys;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MenubarPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn menubar_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "menubar popper vars" (`--radix-menubar-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-menubar-content-available-width`
/// - `--radix-menubar-content-available-height`
/// - `--radix-menubar-trigger-width`
/// - `--radix-menubar-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn menubar_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> MenubarPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    MenubarPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::{Point, Size};

    #[test]
    fn menubar_popper_vars_available_height_tracks_flipped_side_space() {
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
        let vars = menubar_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
    }
}
