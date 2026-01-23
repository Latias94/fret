//! Radix `ContextMenu` facades.
//!
//! Upstream: <https://github.com/radix-ui/primitives/tree/main/packages/react/context-menu>
//!
//! In Radix, `ContextMenu` is built on top of `Menu` with a different trigger/open policy.
//! In Fret we share the same underlying behavior via `crate::primitives::menu` and expose
//! Radix-named entry points here for reuse outside the shadcn layer.

use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{MouseButton, Point, Px, Rect};
use fret_runtime::{Model, ModelId};
use fret_ui::UiHost;
use fret_ui::action::{OnPointerDown, PointerDownCx, UiPointerActionHost};

use crate::primitives::popper;

pub use crate::primitives::menu::*;

pub use crate::primitives::menu::root::dismissible_menu_request as context_menu_dismissible_request;
pub use crate::primitives::menu::root::dismissible_menu_request_with_dismiss_handler as context_menu_dismissible_request_with_dismiss_handler;
pub use crate::primitives::menu::root::menu_overlay_root_name as context_menu_root_name;
pub use crate::primitives::menu::root::with_root_name_sync_root_open_and_ensure_submenu as context_menu_sync_root_open_and_ensure_submenu;
pub use crate::primitives::menu::trigger::wire_open_on_shift_f10 as wire_context_menu_open_on_shift_f10;

#[derive(Default)]
struct ContextMenuAnchorStore {
    by_open_model: Option<Model<HashMap<ModelId, Point>>>,
}

/// Returns a shared anchor store keyed by the context menu's open model id.
///
/// This is intended for context menus that need to anchor by cursor position even when the trigger
/// is not a `PointerRegion` (e.g. viewport tools opened via `Effect::ViewportInput`).
pub fn context_menu_anchor_store_model<H: UiHost>(app: &mut H) -> Model<HashMap<ModelId, Point>> {
    app.with_global_mut_untracked(ContextMenuAnchorStore::default, |st, app| {
        if let Some(model) = st.by_open_model.clone() {
            return model;
        }
        let model = app.models_mut().insert(HashMap::<ModelId, Point>::new());
        st.by_open_model = Some(model.clone());
        model
    })
}

/// Updates the anchor point for the given open model.
pub fn set_context_menu_anchor_for_open_model<H: UiHost>(
    app: &mut H,
    open: &Model<bool>,
    position: Point,
) {
    let open_model_id = open.id();
    let anchor_store_model = context_menu_anchor_store_model(app);
    let _ = app.models_mut().update(&anchor_store_model, |map| {
        map.insert(open_model_id, position);
    });
}

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
///
/// Note: `PointerRegionState::last_down` is per-element state; if you need the anchor to persist
/// across re-renders (or want to decouple it from element identity), copy `down.position` into an
/// app-owned model (e.g. `Model<Option<Point>>`, or a map keyed by your `open` model id).
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
    popper::popper_desired_width(outer, anchor, min_width)
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
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
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
    use fret_core::Size;

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
