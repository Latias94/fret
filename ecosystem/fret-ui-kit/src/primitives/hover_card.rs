//! Hover Card primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx`
//!
//! This module is intentionally thin: it provides Radix-named entry points for overlay root naming
//! and hover overlay request wiring. Visual styling, motion, and arrow rendering belong in higher
//! layers (e.g. shadcn recipes).

use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayRequest};

/// Stable per-overlay root naming convention for hover cards.
pub fn hover_card_root_name(id: GlobalElementId) -> String {
    OverlayController::hover_overlay_root_name(id)
}

/// Builds an overlay request for a Radix-style hover card.
pub fn hover_card_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::hover(id, trigger, children);
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

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;

    #[test]
    fn hover_card_request_sets_default_root_name() {
        let mut app = App::new();
        fret_ui::elements::with_element_cx(
            &mut app,
            Default::default(),
            Default::default(),
            "test",
            |_cx| {
                let id = GlobalElementId(0x123);
                let trigger = GlobalElementId(0x456);
                let req = hover_card_request(id, trigger, Vec::new());
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
}
