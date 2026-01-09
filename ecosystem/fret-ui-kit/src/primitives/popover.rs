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
use fret_ui::action::OnDismissRequest;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::dialog as dialog_prim;
pub use crate::primitives::popper::{Align, LayoutDirection, Side};
use crate::primitives::trigger_a11y;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PopoverVariant {
    #[default]
    NonModal,
    Modal,
}

#[derive(Clone)]
pub struct PopoverOptions {
    pub variant: PopoverVariant,
    pub consume_outside_pointer_events: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub on_dismiss_request: Option<OnDismissRequest>,
}

impl std::fmt::Debug for PopoverOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PopoverOptions")
            .field("variant", &self.variant)
            .field(
                "consume_outside_pointer_events",
                &self.consume_outside_pointer_events,
            )
            .field("initial_focus", &self.initial_focus)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .finish()
    }
}

impl Default for PopoverOptions {
    fn default() -> Self {
        Self {
            variant: PopoverVariant::NonModal,
            consume_outside_pointer_events: false,
            initial_focus: None,
            on_dismiss_request: None,
        }
    }
}

impl PopoverOptions {
    pub fn modal(mut self, modal: bool) -> Self {
        self.variant = if modal {
            PopoverVariant::Modal
        } else {
            PopoverVariant::NonModal
        };
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    pub fn initial_focus(mut self, element: GlobalElementId) -> Self {
        self.initial_focus = Some(element);
        self
    }

    /// Installs a DismissableLayer-style dismiss handler.
    ///
    /// When set, this overrides the default behavior that closes the popover on dismiss.
    /// Callers may update the `open` model themselves (or intentionally keep it open) to mirror
    /// Radix `onInteractOutside` / `onEscapeKeyDown` `preventDefault` outcomes.
    pub fn on_dismiss_request(mut self, handler: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = handler;
        self
    }
}

/// Stable per-overlay root naming convention for popover-like overlays.
pub fn popover_root_name(id: GlobalElementId) -> String {
    OverlayController::popover_root_name(id)
}

/// Stable per-overlay root naming convention for popover-like overlays (modal variant).
pub fn popover_modal_root_name(id: GlobalElementId) -> String {
    OverlayController::modal_root_name(id)
}

/// A Radix-shaped `Popover` root configuration surface.
///
/// Upstream supports a controlled/uncontrolled `open` state (`open` + `defaultOpen`). In Fret this
/// maps to either:
/// - a caller-provided `Model<bool>` (controlled), or
/// - an internal `Model<bool>` stored in element state (uncontrolled).
#[derive(Debug, Clone, Default)]
pub struct PopoverRoot {
    open: Option<Model<bool>>,
    default_open: bool,
    options: PopoverOptions,
}

impl PopoverRoot {
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

    pub fn modal(mut self, modal: bool) -> Self {
        self.options = self.options.modal(modal);
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.options = self.options.consume_outside_pointer_events(consume);
        self
    }

    pub fn initial_focus(mut self, element: GlobalElementId) -> Self {
        self.options = self.options.initial_focus(element);
        self
    }

    pub fn on_dismiss_request(mut self, handler: Option<OnDismissRequest>) -> Self {
        self.options = self.options.on_dismiss_request(handler);
        self
    }

    pub fn options(&self) -> PopoverOptions {
        self.options.clone()
    }

    /// Returns a `Model<bool>` that behaves like Radix `useControllableState` for `open`.
    pub fn use_open_model<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
    ) -> crate::primitives::controllable_state::ControllableModel<bool> {
        popover_use_open_model(cx, self.open.clone(), || self.default_open)
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
/// This is a convenience helper for authoring Radix-shaped popover roots:
/// - if `controlled_open` is provided, it is used directly
/// - otherwise an internal model is created (once) using `default_open` (Radix `defaultOpen`)
pub fn popover_use_open_model<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    controlled_open: Option<Model<bool>>,
    default_open: impl FnOnce() -> bool,
) -> crate::primitives::controllable_state::ControllableModel<bool> {
    crate::primitives::open_state::open_use_model(cx, controlled_open, default_open)
}

/// A minimal semantics wrapper matching Radix `PopoverContent` (`role="dialog"`).
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
    trigger: AnyElement,
    expanded: bool,
    dialog_element: Option<GlobalElementId>,
) -> AnyElement {
    trigger_a11y::apply_trigger_controls_expanded(trigger, Some(expanded), dialog_element)
}

/// Builds an overlay request for a Radix-style popover.
///
/// This supports the two Radix outcomes:
/// - non-modal popover (`variant=NonModal`): click-through by default (ADR 0069)
/// - modal popover (`variant=Modal`): uses the shared modal overlay mechanism
pub fn popover_request(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    options: PopoverOptions,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = match options.variant {
        PopoverVariant::NonModal => {
            OverlayRequest::dismissible_popover(id, trigger, open, presence, children)
        }
        PopoverVariant::Modal => OverlayRequest::modal(id, Some(trigger), open, presence, children),
    };

    request.root_name = Some(match options.variant {
        PopoverVariant::NonModal => popover_root_name(id),
        PopoverVariant::Modal => popover_modal_root_name(id),
    });
    request.consume_outside_pointer_events = options.consume_outside_pointer_events;
    request.initial_focus = options.initial_focus;
    request.dismissible_on_dismiss_request = options.on_dismiss_request;
    request
}

/// Builds an overlay request for a Radix-style popover, adding an optional anchor subtree as a
/// dismissable branch.
///
/// This is a convenience for the common Radix `PopoverAnchor` outcome: the placement anchor may
/// live outside the trigger subtree, but should still be treated as "inside" for outside press /
/// focus outside decisions.
pub fn popover_request_with_anchor(
    id: GlobalElementId,
    trigger: GlobalElementId,
    anchor: Option<GlobalElementId>,
    open: Model<bool>,
    presence: OverlayPresence,
    options: PopoverOptions,
    children: Vec<AnyElement>,
) -> OverlayRequest {
    let mut request = popover_request(id, trigger, open, presence, options, children);
    if let Some(anchor) = anchor
        && anchor != trigger
    {
        request.dismissable_branches.push(anchor);
    }
    request
}

/// Layout used for a Radix-like popover modal barrier element.
///
/// This is a re-export of the shared modal barrier layout from `primitives::dialog`.
pub fn popover_modal_barrier_layout() -> LayoutStyle {
    dialog_prim::modal_barrier_layout()
}

/// Builds a full-window modal barrier for Radix-like popover overlays.
///
/// This is a thin wrapper over `primitives::dialog::modal_barrier` so non-shadcn users can reuse
/// the same "hide others + block outside pointer events" outcome without depending on dialog
/// primitives.
pub fn popover_modal_barrier<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    children: Vec<AnyElement>,
) -> AnyElement {
    dialog_prim::modal_barrier(cx, open, dismiss_on_press, children)
}

/// Convenience helper to assemble modal popover overlay children in a Radix-like order: barrier
/// then content.
///
/// This delegates to the Dialog barrier helpers, since Radix Popover's modal variant shares the
/// same "hide others + block outside pointer events" outcome.
pub fn popover_modal_layer_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    barrier_children: Vec<AnyElement>,
    content: AnyElement,
) -> Vec<AnyElement> {
    vec![
        popover_modal_barrier(cx, open, true, barrier_children),
        content,
    ]
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
    popover_request(
        trigger,
        trigger,
        open,
        presence,
        PopoverOptions::default(),
        children,
    )
}

/// Requests a Radix-style non-modal popover overlay for the current window.
pub fn request_dismissible_popover<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    request: OverlayRequest,
) {
    OverlayController::request(cx, request);
}

/// Requests a Radix-style popover overlay for the current window.
pub fn request_popover<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
    OverlayController::request(cx, request);
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Px, Rect, Size};
    use fret_ui::action::DismissReason;
    use fret_ui::element::{AnyElement, ElementKind, LayoutStyle, PressableProps};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn popover_root_open_model_uses_controlled_model() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        let controlled = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root = PopoverRoot::new()
                .open(Some(controlled.clone()))
                .default_open(false);
            assert_eq!(root.open_model(cx), controlled);
        });
    }

    #[test]
    fn popover_root_options_builder_updates_options() {
        let handler: OnDismissRequest = Arc::new(|_host, _cx, _reason: DismissReason| {});
        let root = PopoverRoot::new()
            .modal(true)
            .consume_outside_pointer_events(true)
            .initial_focus(GlobalElementId(0xbeef))
            .on_dismiss_request(Some(handler.clone()));
        let options = root.options();
        assert_eq!(options.variant, PopoverVariant::Modal);
        assert!(options.consume_outside_pointer_events);
        assert_eq!(options.initial_focus, Some(GlobalElementId(0xbeef)));
        assert!(options.on_dismiss_request.is_some());
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
            let trigger = apply_popover_trigger_a11y(trigger, true, Some(dialog_id));

            let ElementKind::Pressable(PressableProps { a11y, .. }) = &trigger.kind else {
                panic!("expected pressable trigger");
            };
            assert_eq!(a11y.expanded, Some(true));
            assert_eq!(a11y.controls_element, Some(dialog_id.0));
        });
    }

    #[test]
    fn popover_dialog_wrapper_id_matches_rendered_wrapper_id() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let root_name = "popover-dialog-wrapper-id-test";
            let computed = popover_dialog_wrapper_id::<App>(cx, root_name);
            let rendered = cx.with_root_name(root_name, |cx| {
                popover_dialog_wrapper::<App>(cx, None, |_cx| Vec::new())
            });
            assert_eq!(computed, rendered.id);
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

    #[test]
    fn popover_request_propagates_options_and_chooses_kind() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x111);
        let trigger = GlobalElementId(0x222);
        let focus = GlobalElementId(0x333);

        let req = popover_request(
            id,
            trigger,
            open.clone(),
            OverlayPresence::instant(true),
            PopoverOptions::default()
                .consume_outside_pointer_events(true)
                .initial_focus(focus),
            Vec::new(),
        );
        assert_eq!(req.kind, crate::OverlayKind::NonModalDismissible);
        assert_eq!(req.consume_outside_pointer_events, true);
        assert_eq!(req.initial_focus, Some(focus));
        assert_eq!(
            req.root_name.as_deref(),
            Some(popover_root_name(id).as_str())
        );

        let req = popover_request(
            id,
            trigger,
            open,
            OverlayPresence::instant(true),
            PopoverOptions::default().modal(true),
            Vec::new(),
        );
        assert_eq!(req.kind, crate::OverlayKind::Modal);
        assert_eq!(
            req.root_name.as_deref(),
            Some(popover_modal_root_name(id).as_str())
        );
    }

    #[test]
    fn popover_request_with_anchor_adds_dismissable_branch() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);
        let id = GlobalElementId(0x111);
        let trigger = GlobalElementId(0x222);
        let anchor = GlobalElementId(0x333);

        let req = popover_request_with_anchor(
            id,
            trigger,
            Some(anchor),
            open,
            OverlayPresence::instant(true),
            PopoverOptions::default(),
            Vec::new(),
        );
        assert_eq!(req.dismissable_branches, vec![anchor]);
    }

    #[test]
    fn popover_modal_layer_children_wraps_with_barrier() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();
        let open = app.models_mut().insert(true);

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let content: AnyElement = cx.container(Default::default(), |_cx| Vec::new());
            let children =
                popover_modal_layer_children::<App>(cx, open.clone(), Vec::new(), content);
            assert_eq!(children.len(), 2);
        });
    }
}
