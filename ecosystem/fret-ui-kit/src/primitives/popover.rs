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

use fret_core::{Px, Rect};
use fret_runtime::Model;
use fret_ui::action::{OnCloseAutoFocus, OnDismissRequest, OnOpenAutoFocus};
use fret_ui::element::{AnyElement, Elements, LayoutStyle, SemanticsProps};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::declarative::ModelWatchExt;
use crate::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::dialog as dialog_prim;
use crate::primitives::popper;
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
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
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
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .finish()
    }
}

impl Default for PopoverOptions {
    fn default() -> Self {
        Self {
            variant: PopoverVariant::NonModal,
            consume_outside_pointer_events: false,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
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

    pub fn on_open_auto_focus(mut self, hook: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = hook;
        self
    }

    pub fn on_close_auto_focus(mut self, hook: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = hook;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PopoverPopperVars {
    pub available_width: Px,
    pub available_height: Px,
    pub trigger_width: Px,
    pub trigger_height: Px,
}

pub fn popover_popper_desired_width(outer: Rect, anchor: Rect, min_width: Px) -> Px {
    popper::popper_desired_width(outer, anchor, min_width)
}

/// Compute Radix-like "popover popper vars" (`--radix-popover-*`) for recipes.
///
/// Upstream Radix re-namespaces these from `@radix-ui/react-popper`:
/// - `--radix-popover-content-available-width`
/// - `--radix-popover-content-available-height`
/// - `--radix-popover-trigger-width`
/// - `--radix-popover-trigger-height`
///
/// In Fret, we compute the same concepts as a structured return value so recipes can constrain
/// their content without relying on CSS variables.
pub fn popover_popper_vars(
    outer: Rect,
    anchor: Rect,
    min_width: Px,
    placement: popper::PopperContentPlacement,
) -> PopoverPopperVars {
    let metrics =
        popper::popper_available_metrics_for_placement(outer, anchor, min_width, placement);
    PopoverPopperVars {
        available_width: metrics.available_width,
        available_height: metrics.available_height,
        trigger_width: metrics.anchor_width,
        trigger_height: metrics.anchor_height,
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

    pub fn options(&self) -> PopoverOptions {
        self.options.clone()
    }

    pub fn request_with_dismiss_handler<H: UiHost>(
        &self,
        cx: &mut ElementContext<'_, H>,
        id: GlobalElementId,
        trigger: GlobalElementId,
        presence: OverlayPresence,
        on_dismiss_request: Option<OnDismissRequest>,
        children: Vec<AnyElement>,
    ) -> OverlayRequest {
        popover_request_with_dismiss_handler(
            id,
            trigger,
            self.open_model(cx),
            presence,
            self.options.clone(),
            on_dismiss_request,
            children,
        )
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
pub fn popover_dialog_wrapper<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    label: Option<Arc<str>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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
        let element = popover_dialog_wrapper(cx, None, |_cx| Vec::new());
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
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let children: Vec<AnyElement> = children.into_iter().collect();
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
    request.on_open_auto_focus = options.on_open_auto_focus.clone();
    request.on_close_auto_focus = options.on_close_auto_focus.clone();
    request
}

/// Builds an overlay request for a Radix-style popover, with an explicit dismiss handler.
///
/// This mirrors the Radix `DismissableLayer` contract: callers may "prevent default" by not
/// closing the `open` model in the handler.
pub fn popover_request_with_dismiss_handler(
    id: GlobalElementId,
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    options: PopoverOptions,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let mut request = popover_request(id, trigger, open, presence, options, children);
    request.dismissible_on_dismiss_request = on_dismiss_request;
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
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let mut request = popover_request(id, trigger, open, presence, options, children);
    if let Some(anchor) = anchor
        && anchor != trigger
    {
        request = request.add_dismissable_branch(anchor);
    }
    request
}

/// Builds an overlay request for a Radix-style popover, adding an optional anchor subtree and a
/// custom dismiss handler.
pub fn popover_request_with_anchor_and_dismiss_handler(
    id: GlobalElementId,
    trigger: GlobalElementId,
    anchor: Option<GlobalElementId>,
    open: Model<bool>,
    presence: OverlayPresence,
    options: PopoverOptions,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> OverlayRequest {
    let mut request =
        popover_request_with_anchor(id, trigger, anchor, open, presence, options, children);
    request.dismissible_on_dismiss_request = on_dismiss_request;
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
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    popover_modal_barrier_with_dismiss_handler(cx, open, dismiss_on_press, None, children)
}

/// Builds a full-window modal barrier for Radix-like popover overlays, routing presses through an
/// optional dismiss handler.
pub fn popover_modal_barrier_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    dismiss_on_press: bool,
    on_dismiss_request: Option<OnDismissRequest>,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement {
    dialog_prim::modal_barrier_with_dismiss_handler(
        cx,
        open,
        dismiss_on_press,
        on_dismiss_request,
        children,
    )
}

/// Convenience helper to assemble modal popover overlay children in a Radix-like order: barrier
/// then content.
///
/// This delegates to the Dialog barrier helpers, since Radix Popover's modal variant shares the
/// same "hide others + block outside pointer events" outcome.
pub fn popover_modal_layer_elements<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        popover_modal_barrier(cx, open, true, barrier_children),
        content,
    ])
}

/// Convenience helper to assemble modal popover overlay children in a Radix-like order (barrier
/// then content), while routing barrier presses through an optional dismiss handler.
pub fn popover_modal_layer_elements_with_dismiss_handler<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    on_dismiss_request: Option<OnDismissRequest>,
    barrier_children: impl IntoIterator<Item = AnyElement>,
    content: AnyElement,
) -> Elements {
    Elements::from([
        popover_modal_barrier_with_dismiss_handler(
            cx,
            open,
            true,
            on_dismiss_request,
            barrier_children,
        ),
        content,
    ])
}

/// Builds an overlay request for a Radix-style non-modal popover.
///
/// This is click-through by default (outside press closes the popover but still allows underlay
/// hit-tested dispatch), matching the typical Radix Popover behavior (ADR 0069).
pub fn dismissible_popover_request(
    trigger: GlobalElementId,
    open: Model<bool>,
    presence: OverlayPresence,
    children: impl IntoIterator<Item = AnyElement>,
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
    use fret_core::Event;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService};
    use fret_ui::UiTree;
    use fret_ui::action::DismissReason;
    use fret_ui::element::{
        AnyElement, ElementKind, InsetStyle, LayoutStyle, Length, PositionStyle, PressableProps,
        SizeStyle,
    };

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
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
        let root = PopoverRoot::new()
            .modal(true)
            .consume_outside_pointer_events(true)
            .initial_focus(GlobalElementId(0xbeef));
        let options = root.options();
        assert_eq!(options.variant, PopoverVariant::Modal);
        assert!(options.consume_outside_pointer_events);
        assert_eq!(options.initial_focus, Some(GlobalElementId(0xbeef)));
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
    fn popover_popper_vars_available_height_tracks_flipped_side_space() {
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
        let vars = popover_popper_vars(outer, anchor, Px(0.0), placement);
        assert!(vars.available_height.0 > 60.0 && vars.available_height.0 < 80.0);
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
                popover_dialog_wrapper(cx, None, |_cx| Vec::new())
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
    fn popover_request_with_dismiss_handler_sets_dismiss_handler() {
        let mut app = App::new();
        let open = app.models_mut().insert(false);

        let handler: OnDismissRequest =
            Arc::new(|_host, _cx, _req: &mut fret_ui::action::DismissRequestCx| {});
        let req = popover_request_with_dismiss_handler(
            GlobalElementId(0x123),
            GlobalElementId(0x123),
            open,
            OverlayPresence::instant(true),
            PopoverOptions::default(),
            Some(handler),
            Vec::new(),
        );

        assert!(req.dismissible_on_dismiss_request.is_some());
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
                popover_modal_layer_elements::<App>(cx, open.clone(), [], content).into_vec();
            assert_eq!(children.len(), 2);
        });
    }

    #[test]
    fn popover_modal_barrier_press_can_be_intercepted() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let b = bounds();

        OverlayController::begin_frame(&mut app, window);
        let trigger_cell = std::cell::Cell::new(None);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            b,
            "base",
            |cx| {
                vec![cx.pressable_with_id(
                    PressableProps {
                        layout: LayoutStyle::default(),
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_cell.set(Some(id));
                        Vec::new()
                    },
                )]
            },
        );
        ui.set_root(base);

        let open = app.models_mut().insert(true);
        let popover_id = GlobalElementId(0xabc);
        let trigger = trigger_cell.get().expect("trigger id");

        let reason_cell: std::sync::Arc<std::sync::Mutex<Option<DismissReason>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let reason_cell_for_handler = reason_cell.clone();
        let handler: OnDismissRequest = Arc::new(move |_host, _cx, req| {
            *reason_cell_for_handler.lock().expect("reason lock") = Some(req.reason);
            req.prevent_default();
        });

        let overlay_children =
            fret_ui::elements::with_element_cx(&mut app, window, b, "popover", |cx| {
                // Place content away from the click point so barrier presses can be observed.
                let content: AnyElement = cx.pressable(
                    PressableProps {
                        layout: LayoutStyle {
                            position: PositionStyle::Absolute,
                            inset: InsetStyle {
                                top: Some(Px(80.0)),
                                left: Some(Px(160.0)),
                                right: None,
                                bottom: None,
                            },
                            size: SizeStyle {
                                width: Length::Px(Px(20.0)),
                                height: Length::Px(Px(20.0)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        enabled: true,
                        focusable: false,
                        ..Default::default()
                    },
                    |_cx, _st| Vec::new(),
                );
                popover_modal_layer_elements_with_dismiss_handler::<App>(
                    cx,
                    open.clone(),
                    Some(handler.clone()),
                    [],
                    content,
                )
                .into_vec()
            });

        let req = popover_request(
            popover_id,
            trigger,
            open.clone(),
            OverlayPresence::instant(true),
            PopoverOptions::default().modal(true),
            overlay_children,
        );
        OverlayController::request_for_window(&mut app, window, req);
        OverlayController::render(&mut ui, &mut app, &mut services, window, b);
        ui.layout_all(&mut app, &mut services, b, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: Default::default(),
            }),
        );

        assert_eq!(app.models().get_copied(&open), Some(true));
        let reason = *reason_cell.lock().expect("reason lock");
        let Some(DismissReason::OutsidePress { pointer }) = reason else {
            panic!("expected outside-press dismissal, got {reason:?}");
        };
        let Some(cx) = pointer else {
            panic!("expected pointer payload for outside-press dismissal");
        };
        assert_eq!(cx.pointer_id, fret_core::PointerId(0));
        assert_eq!(cx.pointer_type, fret_core::PointerType::Mouse);
        assert_eq!(cx.button, fret_core::MouseButton::Left);
        assert_eq!(cx.modifiers, fret_core::Modifiers::default());
        assert_eq!(cx.click_count, 1);
    }
}
