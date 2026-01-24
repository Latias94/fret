//! Hover Card primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx`
//!
//! This module is intentionally thin: it provides Radix-named entry points for overlay root naming
//! and hover overlay request wiring. Visual styling, motion, and arrow rendering belong in higher
//! layers (e.g. shadcn recipes).

use fret_core::{Px, Rect};
use fret_runtime::Model;
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::primitives::popper;
use crate::{OverlayController, OverlayRequest};

/// Stable per-overlay root naming convention for hover cards.
pub fn hover_card_root_name(id: GlobalElementId) -> String {
    OverlayController::hover_overlay_root_name(id)
}

/// A Radix-shaped `HoverCard` root configuration surface (open state only).
///
/// Radix HoverCard supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In
/// Fret, hover-card recipes often derive open state from hover intent, but this root helper keeps
/// a Radix-shaped option available for non-hover use cases and for strict parity tests.
#[derive(Debug, Clone, Default)]
pub struct HoverCardRoot {
    open: Option<Model<bool>>,
    default_open: bool,
}

impl HoverCardRoot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the controlled `open` model (`Some`) or selects uncontrolled mode (`None`).
    pub fn open(mut self, open: Option<Model<bool>>) -> Self {
        self.open = open;
        self
    }

    /// Sets the uncontrolled initial open value (Radix `defaultOpen`).
    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        hover_card_use_open_model(cx, self.open.clone(), || self.default_open)
    }

    pub fn open_model<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> Model<bool> {
        self.use_open_model(cx).model()
    }

    /// Reads the current open value from the derived open model.
    pub fn is_open<H: UiHost>(&self, cx: &mut ElementContext<'_, H>) -> bool {
        let open_model = self.open_model(cx);
        cx.watch_model(&open_model)
            .layout()
            .copied()
            .unwrap_or(false)
    }
}

/// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
///
/// This is a convenience helper for authoring Radix-shaped hover-card roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn hover_card_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// Builds an overlay request for a Radix-style hover card.
pub fn hover_card_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: crate::OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::hover(id, trigger, open, presence, children);
    request.root_name = Some(hover_card_root_name(id));
    request
}

/// Requests a hover-card overlay for the current window.
pub fn request_hover_card<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

/// Computes whether the hover card should be considered "hovered" for intent/visibility decisions.
///
/// Notes:
/// - Pointer hover is level-triggered: `trigger_hovered || overlay_hovered`.
/// - Keyboard focus should be treated as an "open affordance" for accessibility flows. In Radix,
///   pointer-driven focus (mouse down) does *not* keep the hover card open after pointer leave.
///   Call sites should pass `keyboard_focused` (not just `focused`).
pub fn hover_card_hovered(
    trigger_hovered: bool,
    overlay_hovered: bool,
    keyboard_focused: bool,
) -> bool {
    trigger_hovered || overlay_hovered || keyboard_focused
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HoverCardPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn hover_card_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "hover card popper vars" (`--radix-hover-card-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-hover-card-content-available-width`
/// - `--radix-hover-card-content-available-height`
/// - `--radix-hover-card-trigger-width`
/// - `--radix-hover-card-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn hover_card_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> HoverCardPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    HoverCardPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{Point, Size};

    #[test]
    fn hover_card_root_open_model_uses_controlled_model() {
        let window = Default::default();
        let mut app = App::new();

        let controlled = app.models_mut().insert(true);
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let root = HoverCardRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn hover_card_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(true);
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            move |_cx| {
                let id = GlobalElementId(0x123);
                let trigger = GlobalElementId(0x456);
                let req = hover_card_request(
                    id,
                    trigger,
                    open.clone(),
                    crate::OverlayPresence::instant(true),
                    Vec::new(),
                );
                let expected = hover_card_root_name(id);
                assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
            },
        );
    }

    #[test]
    fn hover_card_hovered_or_logic_matches_expectations() {
        assert!(!hover_card_hovered(false, false, false));
        assert!(hover_card_hovered(true, false, false));
        assert!(hover_card_hovered(false, true, false));
        assert!(hover_card_hovered(false, false, true));
    }

    #[test]
    fn hover_card_popper_vars_available_height_tracks_flipped_side_space() {
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
        let vars = hover_card_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
    }
}
