//! Popover helpers (Radix `@radix-ui/react-popover` outcomes).
//!
//! Upstream Popover composes:
//! - anchored floating placement (`@radix-ui/react-popper`)
//! - conditional mounting (`@radix-ui/react-presence`)
//! - dismissal + focus management (`@radix-ui/react-dismissable-layer`, `@radix-ui/react-focus-scope`)
//! - portal rendering (`@radix-ui/react-portal`)
//!
//! In Fret, these concerns map to:
//! - placement: `crate::primitives::popper` / `crate::primitives::popper_content`
//! - presence: `crate::OverlayPresence` (driven by motion helpers in recipe layers)
//! - portal + dismissal + focus: per-window overlay roots (`crate::OverlayController`)
//!
//! This module is intentionally thin: it provides Radix-named entry points for a11y stamping and
//! overlay request wiring without forcing a visual skin.

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind, PressableProps, SemanticsProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

pub use crate::primitives::popper::{Align, LayoutDirection, Side};

/// Stable per-overlay root naming convention for popover-like overlays.
pub fn popover_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// A minimal semantics wrapper matching Radix `PopoverContent` (`role="dialog"`).
#[track_caller]
pub fn popover_dialog_wrapper<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Option<Arc<str>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    cx.semantics_with_id(
        SemanticsProps {
            role: fret_core::SemanticsRole::Dialog,
            label,
            ..Default::default()
        },
        move |cx, _id| f(cx),
    )
}

/// Returns a stable element id for the popover content "dialog" wrapper.
///
/// This is intended for `aria-controls` / `controls_element` style relationships:
/// the trigger can reference this element to indicate which dialog/panel it controls.
#[track_caller]
pub fn popover_dialog_wrapper_id<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: &str,
) -> GlobalElementId {
    cx.with_root_name(overlay_root_name, |cx| {
        let element = popover_dialog_wrapper::<H>(cx, None, |_cx| Vec::new());
        element.id
    })
}

/// Stamps Radix-like trigger semantics:
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_popover_trigger_a11y(
    mut trigger: AnyElement,
    expanded: bool,
    dialog_element: GlobalElementId,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            a11y.expanded = Some(expanded);
            a11y.controls_element = Some(dialog_element.0);
        }
        ElementKind::Semantics(props) => {
            props.expanded = Some(expanded);
            props.controls_element = Some(dialog_element.0);
        }
        _ => {}
    }
    trigger
}

/// Builds an overlay request for a Radix-style non-modal popover.
///
/// This is click-through by default (outside press closes the popover but still allows underlay
/// hit-tested dispatch), matching the typical Radix Popover behavior (ADR 0069).
pub fn dismissible_popover_request(
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request =
        OverlayRequest::dismissible_popover(trigger, trigger, open, presence, children);
    request.root_name = Some(popover_root_name(trigger));
    request
}

/// Requests a Radix-style non-modal popover overlay for the current window.
pub fn request_dismissible_popover<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: OverlayRequest,
) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{ElementKind, LayoutStyle, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn apply_popover_trigger_a11y_sets_controls_and_expanded() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let trigger = cx.pressable(
                PressableProps {
                    layout: LayoutStyle::default(),
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let dialog_id = popover_dialog_wrapper_id::<App>(cx, "popover-a11y-test");
            let trigger = apply_popover_trigger_a11y(trigger, true, dialog_id);

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(dialog_id.0));
        });
    }

    #[test]
    fn dismissible_popover_request_sets_default_root_name() {
        let mut app = App::new();
        let model = app.models_mut().insert(false);

        let req = dismissible_popover_request(
            GlobalElementId(0x123),
            model,
            OverlayPresence::instant(true),
            Vec::new(),
        );
        let expected = popover_root_name(GlobalElementId(0x123));
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }
}
