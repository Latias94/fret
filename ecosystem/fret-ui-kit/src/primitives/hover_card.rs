//! Hover Card primitives (Radix-aligned outcomes).
//!
//! Upstream reference:
//! - `repo-ref/primitives/packages/react/hover-card/src/hover-card.tsx`
//!
//! This module is intentionally thin: it provides Radix-named entry points for overlay root naming
//! and hover overlay request wiring. Visual styling, motion, and arrow rendering belong in higher
//! layers (e.g. shadcn recipes).

use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayRequest};

/// Stamps Radix-like trigger relationships:
/// - `described_by_element` mirrors `aria-describedby` (by element id).
///
/// Radix HoverCard sets `aria-describedby` on the trigger while open to associate it with the
/// hover-card content. In Fret we model this via a portable element-id relationship that resolves
/// into `SemanticsNode.described_by` when the hover-card content is mounted.
pub fn apply_hover_card_trigger_a11y(
    mut trigger: AnyElement,
    content_element: GlobalElementId,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(props) => {
            props.a11y.described_by_element = Some(content_element.0);
        }
        ElementKind::Semantics(props) => {
            props.described_by_element = Some(content_element.0);
        }
        _ => {}
    }
    trigger
}

/// Stable per-overlay root naming convention for hover cards.
pub fn hover_card_root_name(id: GlobalElementId) -> String {
    OverlayController::hover_overlay_root_name(id)
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
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps, SemanticsProps};

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
    fn apply_hover_card_trigger_a11y_sets_described_by_on_pressable() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );
            let content = GlobalElementId(0xbeef);
            let trigger = apply_hover_card_trigger_a11y(trigger, content);
            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable");
            };
            assert_eq!(a11y.described_by_element, Some(content.0));
        });
    }

    #[test]
    fn apply_hover_card_trigger_a11y_sets_described_by_on_semantics() {
        let window = Default::default();
        let mut app = App::new();
        fret_ui::elements::with_element_cx(&mut app, window, Default::default(), "test", |cx| {
            let trigger = cx.semantics(SemanticsProps::default(), |_cx| Vec::new());
            let content = GlobalElementId(0xbeef);
            let trigger = apply_hover_card_trigger_a11y(trigger, content);
            let ElementKind::Semantics(props) = &trigger.kind else {
                panic!("expected semantics");
            };
            assert_eq!(props.described_by_element, Some(content.0));
        });
    }

    #[test]
    fn hover_card_hovered_or_logic_matches_expectations() {
        assert!(!hover_card_hovered(false, false, false));
        assert!(hover_card_hovered(true, false, false));
        assert!(hover_card_hovered(false, true, false));
        assert!(hover_card_hovered(false, false, true));
    }
}
