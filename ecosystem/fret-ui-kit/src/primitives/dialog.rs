//! Dialog helpers (Radix `@radix-ui/react-dialog` outcomes).
//!
//! Upstream Dialog composes:
//! - conditional mounting (`@radix-ui/react-presence`)
//! - portal rendering (`@radix-ui/react-portal`)
//! - dismissal + focus management (`@radix-ui/react-dismissable-layer`, `@radix-ui/react-focus-scope`)
//! - modal scrolling + aria hiding (`react-remove-scroll`, `aria-hidden`)
//!
//! In Fret, these concerns map to:
//! - presence: `crate::OverlayPresence` (driven by motion helpers in recipe layers)
//! - portal + dismissal + focus restore/initial focus: per-window overlays (`crate::OverlayController`)
//! - focus traversal scoping: modal barrier layers in `fret-ui` (ADR 0068)
//!
//! This module is intentionally thin: it provides Radix-named entry points for trigger a11y and
//! modal overlay request wiring, without forcing a visual skin.

use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind, PressableProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for dialog-like modal overlays.
pub fn dialog_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// Stamps Radix-like trigger semantics:
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_dialog_trigger_a11y(
    mut trigger: AnyElement,
    expanded: bool,
    content_element: Option<GlobalElementId>,
) -> AnyElement {
    let Some(content_element) = content_element else {
        return trigger;
    };
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            a11y.expanded = Some(expanded);
            a11y.controls_element = Some(content_element.0);
        }
        ElementKind::Semantics(props) => {
            props.expanded = Some(expanded);
            props.controls_element = Some(content_element.0);
        }
        _ => {}
    }
    trigger
}

/// Builds an overlay request for a Radix-style modal dialog.
pub fn modal_dialog_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::modal(id, Some(trigger), open, presence, children);
    request.root_name = Some(dialog_root_name(id));
    request
}

/// Requests a Radix-style modal dialog overlay for the current window.
pub fn request_modal_dialog<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::element::{LayoutStyle, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn apply_dialog_trigger_a11y_sets_controls_and_expanded() {
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

            let content = GlobalElementId(0xdead);
            let trigger = apply_dialog_trigger_a11y(trigger, true, Some(content));

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(content.0));
        });
    }

    #[test]
    fn modal_dialog_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);

        let req = modal_dialog_request(id, trigger, open, OverlayPresence::instant(true), Vec::new());
        let expected = dialog_root_name(id);
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }
}
