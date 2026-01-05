//! Select helpers (Radix `@radix-ui/react-select` outcomes).
//!
//! Upstream Select composes:
//! - anchored floating placement (`@radix-ui/react-popper`)
//! - portal rendering (`@radix-ui/react-portal`)
//! - focus management + outside interaction blocking (`@radix-ui/react-focus-scope`, `DismissableLayer`)
//! - aria hiding + scroll lock while open (`aria-hidden`, `react-remove-scroll`)
//! - trigger open keys + typeahead selection while closed.
//!
//! In Fret, the "blocking outside interaction" outcome is typically modeled by installing the
//! select content in a modal overlay layer (barrier-backed) while keeping the content semantics
//! as `ListBox` rather than `Dialog`.
//!
//! This module is intentionally thin: it provides Radix-named entry points for trigger a11y and
//! overlay request wiring without forcing a visual skin.

use std::sync::Arc;

use fret_core::KeyCode;
use fret_runtime::Model;
use fret_ui::element::{AnyElement, ElementKind, PressableA11y, PressableProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::{OverlayController, OverlayPresence, OverlayRequest};

/// Stable per-overlay root naming convention for select overlays.
pub fn select_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// Stamps Radix-like trigger semantics:
/// - `role=ComboBox`
/// - `expanded` mirrors `aria-expanded`
/// - `controls_element` mirrors `aria-controls` (by element id).
pub fn apply_select_trigger_a11y(
    mut trigger: AnyElement,
    expanded: bool,
    label: Option<Arc<str>>,
    listbox_element: Option<GlobalElementId>,
) -> AnyElement {
    match &mut trigger.kind {
        ElementKind::Pressable(PressableProps { a11y, .. }) => {
            *a11y = PressableA11y {
                role: Some(fret_core::SemanticsRole::ComboBox),
                label,
                expanded: Some(expanded),
                controls_element: listbox_element.map(|id| id.0),
                ..a11y.clone()
            };
        }
        ElementKind::Semantics(props) => {
            props.role = fret_core::SemanticsRole::ComboBox;
            props.label = label;
            props.expanded = Some(expanded);
            props.controls_element = listbox_element.map(|id| id.0);
        }
        _ => {}
    }
    trigger
}

/// Radix Select trigger "open keys" (`OPEN_KEYS`).
pub fn is_select_open_key(key: KeyCode) -> bool {
    matches!(
        key,
        KeyCode::Space | KeyCode::Enter | KeyCode::ArrowUp | KeyCode::ArrowDown
    )
}

/// Returns `true` when the open key is expected to also produce a click/activate event on key-up.
pub fn select_open_key_suppresses_activate(key: KeyCode) -> bool {
    matches!(key, KeyCode::Space | KeyCode::Enter)
}

/// Builds an overlay request for a Radix-style select content overlay.
///
/// This uses a modal overlay layer to approximate Radix Select's outside interaction blocking.
pub fn modal_select_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = OverlayRequest::modal(id, Some(trigger), open, presence, children);
    request.root_name = Some(select_root_name(id));
    request
}

/// Requests a select overlay for the current window.
pub fn request_select<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
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
    fn apply_select_trigger_a11y_sets_role_expanded_and_controls() {
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

            let listbox = GlobalElementId(0xbeef);
            let trigger = apply_select_trigger_a11y(
                trigger,
                true,
                Some(Arc::from("Select")),
                Some(listbox),
            );

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.role, Some(fret_core::SemanticsRole::ComboBox));
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(listbox.0));
            assert_eq!(a11y.label.as_deref(), Some("Select"));
        });
    }

    #[test]
    fn modal_select_request_sets_default_root_name() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x123);
        let trigger = GlobalElementId(0x456);

        let req = modal_select_request(id, trigger, open, OverlayPresence::instant(true), Vec::new());
        let expected = select_root_name(id);
        assert_eq!(req.root_name.as_deref(), Some(expected.as_str()));
    }

    #[test]
    fn select_open_keys_match_radix_defaults() {
        assert!(is_select_open_key(KeyCode::Enter));
        assert!(is_select_open_key(KeyCode::Space));
        assert!(is_select_open_key(KeyCode::ArrowDown));
        assert!(is_select_open_key(KeyCode::ArrowUp));
        assert!(!is_select_open_key(KeyCode::Escape));

        assert!(select_open_key_suppresses_activate(KeyCode::Enter));
        assert!(select_open_key_suppresses_activate(KeyCode::Space));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowDown));
        assert!(!select_open_key_suppresses_activate(KeyCode::ArrowUp));
    }
}
