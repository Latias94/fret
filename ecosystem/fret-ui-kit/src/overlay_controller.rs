use fret_core::{AppWindowId, Rect};
use fret_runtime::Model;
use fret_ui::action::{
    OnCloseAutoFocus, OnDismissRequest, OnDismissiblePointerMove, OnOpenAutoFocus,
};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost, UiTree};

use crate::headless::presence::PresenceOutput;
use crate::headless::transition::TransitionOutput;
use crate::primitives::presence;
use crate::window_overlays;

/// Presence state for an overlay root (mount/paint vs interactive).
///
/// This is intentionally a small, typed wrapper so component code doesn't pass raw `present: bool`
/// around and accidentally conflate it with `open`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OverlayPresence {
    pub present: bool,
    pub interactive: bool,
}

impl OverlayPresence {
    pub fn hidden() -> Self {
        Self {
            present: false,
            interactive: false,
        }
    }

    pub fn instant(open: bool) -> Self {
        Self {
            present: open,
            interactive: open,
        }
    }

    pub fn from_fade(open: bool, presence: PresenceOutput) -> Self {
        Self {
            present: presence.present,
            interactive: open,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    NonModalDismissible,
    Modal,
    Tooltip,
    Hover,
    ToastLayer,
}

#[derive(Debug, Clone)]
pub struct ToastLayerSpec {
    pub store: Model<window_overlays::ToastStore>,
    pub position: window_overlays::ToastPosition,
    pub style: window_overlays::ToastLayerStyle,
    pub margin: Option<fret_core::Px>,
    pub gap: Option<fret_core::Px>,
    pub toast_min_width: Option<fret_core::Px>,
    pub toast_max_width: Option<fret_core::Px>,
}

#[derive(Clone)]
pub struct OverlayRequest {
    pub kind: OverlayKind,
    pub id: GlobalElementId,
    pub root_name: Option<String>,
    pub trigger: Option<GlobalElementId>,
    /// Extra subtrees that should be treated as "inside" for DismissableLayer-style dismissal.
    ///
    /// This is used to align Radix `DismissableLayerBranch` outcomes across disjoint subtrees.
    pub dismissable_branches: Vec<GlobalElementId>,
    /// When an outside-press observer is dispatched for this overlay, suppress normal hit-tested
    /// pointer-down dispatch to underlay widgets for the same event.
    pub consume_outside_pointer_events: bool,
    /// When true, pointer events outside the overlay subtree should not reach underlay widgets
    /// while the overlay is open (Radix `disableOutsidePointerEvents` outcome).
    pub disable_outside_pointer_events: bool,
    /// Whether this overlay should close when the OS window loses focus.
    pub close_on_window_focus_lost: bool,
    /// Whether this overlay should close when the OS window is resized (or scale factor changes).
    pub close_on_window_resize: bool,
    pub open: Option<Model<bool>>,
    pub dismissible_on_dismiss_request: Option<OnDismissRequest>,
    pub dismissible_on_pointer_move: Option<OnDismissiblePointerMove>,
    pub presence: OverlayPresence,
    pub initial_focus: Option<GlobalElementId>,
    /// Optional Radix-like auto-focus hooks for overlay mount/unmount.
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
    pub children: Vec<AnyElement>,
    pub toast_layer: Option<ToastLayerSpec>,
}

impl std::fmt::Debug for OverlayRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OverlayRequest")
            .field("kind", &self.kind)
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("dismissable_branches_len", &self.dismissable_branches.len())
            .field(
                "consume_outside_pointer_events",
                &self.consume_outside_pointer_events,
            )
            .field(
                "disable_outside_pointer_events",
                &self.disable_outside_pointer_events,
            )
            .field("open", &self.open)
            .field(
                "dismissible_on_dismiss_request",
                &self.dismissible_on_dismiss_request.is_some(),
            )
            .field(
                "dismissible_on_pointer_move",
                &self.dismissible_on_pointer_move.is_some(),
            )
            .field("presence", &self.presence)
            .field("initial_focus", &self.initial_focus)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("children_len", &self.children.len())
            .field("toast_layer", &self.toast_layer)
            .finish()
    }
}

impl OverlayRequest {
    pub fn dismissible_popover(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::NonModalDismissible,
            id,
            root_name: None,
            trigger: Some(trigger),
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: Some(open),
            dismissible_on_dismiss_request: None,
            dismissible_on_pointer_move: None,
            presence,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            children: children.into_iter().collect(),
            toast_layer: None,
        }
    }

    /// Dismissible overlay with non-click-through outside press behavior.
    ///
    /// This matches Radix-aligned "menu-like" overlays where an outside click closes the overlay
    /// without activating the underlay (ADR 0069).
    pub fn dismissible_menu(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        let mut req = Self::dismissible_popover(id, trigger, open, presence, children);
        req.consume_outside_pointer_events = true;
        req.disable_outside_pointer_events = true;
        req
    }

    pub fn modal(
        id: GlobalElementId,
        trigger: Option<GlobalElementId>,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::Modal,
            id,
            root_name: None,
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: Some(open),
            dismissible_on_dismiss_request: None,
            dismissible_on_pointer_move: None,
            presence,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            children: children.into_iter().collect(),
            toast_layer: None,
        }
    }

    pub fn tooltip(
        id: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::Tooltip,
            id,
            root_name: None,
            trigger: None,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: Some(open),
            dismissible_on_dismiss_request: None,
            dismissible_on_pointer_move: None,
            presence,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            children: children.into_iter().collect(),
            toast_layer: None,
        }
    }

    pub fn hover(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self::hover_with_presence(id, trigger, open, presence, children)
    }

    pub fn hover_with_presence(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        presence: OverlayPresence,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            kind: OverlayKind::Hover,
            id,
            root_name: None,
            trigger: Some(trigger),
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: Some(open),
            dismissible_on_dismiss_request: None,
            dismissible_on_pointer_move: None,
            presence,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            children: children.into_iter().collect(),
            toast_layer: None,
        }
    }

    pub fn toast_layer(id: GlobalElementId, store: Model<window_overlays::ToastStore>) -> Self {
        Self {
            kind: OverlayKind::ToastLayer,
            id,
            root_name: None,
            trigger: None,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open: None,
            dismissible_on_dismiss_request: None,
            dismissible_on_pointer_move: None,
            presence: OverlayPresence::hidden(),
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            children: Vec::new(),
            toast_layer: Some(ToastLayerSpec {
                store,
                position: window_overlays::ToastPosition::default(),
                style: window_overlays::ToastLayerStyle::default(),
                margin: None,
                gap: None,
                toast_min_width: None,
                toast_max_width: None,
            }),
        }
    }

    pub fn close_on_window_focus_lost(mut self, close: bool) -> Self {
        self.close_on_window_focus_lost = close;
        self
    }

    pub fn close_on_window_resize(mut self, close: bool) -> Self {
        self.close_on_window_resize = close;
        self
    }

    pub fn toast_position(mut self, position: window_overlays::ToastPosition) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_position requires a ToastLayer request");
        spec.position = position;
        self
    }

    pub fn toast_style(mut self, style: window_overlays::ToastLayerStyle) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_style requires a ToastLayer request");
        spec.style = style;
        self
    }

    pub fn toast_margin(mut self, margin: fret_core::Px) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_margin requires a ToastLayer request");
        spec.margin = Some(margin);
        self
    }

    pub fn toast_gap(mut self, gap: fret_core::Px) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_gap requires a ToastLayer request");
        spec.gap = Some(gap);
        self
    }

    pub fn toast_min_width(mut self, width: fret_core::Px) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_min_width requires a ToastLayer request");
        spec.toast_min_width = Some(width);
        self
    }

    pub fn toast_max_width(mut self, width: fret_core::Px) -> Self {
        let spec = self
            .toast_layer
            .as_mut()
            .expect("toast_max_width requires a ToastLayer request");
        spec.toast_max_width = Some(width);
        self
    }

    pub fn dismissable_branches(mut self, branches: Vec<GlobalElementId>) -> Self {
        self.dismissable_branches = branches;
        self
    }

    pub fn consume_outside_pointer_events(mut self, consume: bool) -> Self {
        self.consume_outside_pointer_events = consume;
        self
    }

    pub fn disable_outside_pointer_events(mut self, disable: bool) -> Self {
        self.disable_outside_pointer_events = disable;
        self
    }
}

/// A small, stable facade over `window_overlays` to keep overlay policy wiring out of shadcn code.
pub struct OverlayController;

/// Snapshot of overlay-related input arbitration state for a single `UiTree`.
///
/// This is intended for ecosystem integration points (docking, viewport tooling, policies) that
/// need a stable way to reason about "what input gating is currently active" without depending on
/// `window_overlays` internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OverlayArbitrationSnapshot {
    /// Whether any non-base overlay layers are visible.
    pub has_any_overlays: bool,
    /// Whether a modal barrier is currently active (`blocks_underlay_input=true` on a visible layer).
    pub modal_barrier_active: bool,
    /// Effective pointer occlusion outcome (Radix `disableOutsidePointerEvents` style gating).
    ///
    /// When `modal_barrier_active=true`, this is always `PointerOcclusion::None` since the modal
    /// barrier already blocks underlay pointer routing.
    pub pointer_occlusion: fret_ui::tree::PointerOcclusion,
    /// Whether any pointer is currently captured by the runtime.
    pub pointer_capture_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayStackEntryKind {
    Base,
    Popover,
    Modal,
    Tooltip,
    Hover,
    ToastLayer,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowOverlayStackEntry {
    pub kind: OverlayStackEntryKind,
    pub id: Option<GlobalElementId>,
    pub open: bool,
    pub visible: bool,
    pub blocks_underlay_input: bool,
    pub hit_testable: bool,
    pub pointer_occlusion: fret_ui::tree::PointerOcclusion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowOverlayStackSnapshot {
    pub arbitration: OverlayArbitrationSnapshot,
    pub stack: Vec<WindowOverlayStackEntry>,
    pub topmost_overlay: Option<GlobalElementId>,
    pub topmost_popover: Option<GlobalElementId>,
    pub topmost_modal: Option<GlobalElementId>,
    pub topmost_pointer_occluding_overlay: Option<GlobalElementId>,
}

impl OverlayController {
    pub fn begin_frame<H: UiHost>(app: &mut H, window: AppWindowId) {
        window_overlays::begin_frame(app, window);
    }

    pub fn popover_root_name(id: GlobalElementId) -> String {
        window_overlays::popover_root_name(id)
    }

    pub fn modal_root_name(id: GlobalElementId) -> String {
        window_overlays::modal_root_name(id)
    }

    pub fn tooltip_root_name(id: GlobalElementId) -> String {
        window_overlays::tooltip_root_name(id)
    }

    pub fn hover_overlay_root_name(id: GlobalElementId) -> String {
        window_overlays::hover_overlay_root_name(id)
    }

    pub fn toast_layer_root_name(id: GlobalElementId) -> String {
        window_overlays::toast_layer_root_name(id)
    }

    pub fn request<H: UiHost>(cx: &mut ElementContext<'_, H>, request: OverlayRequest) {
        Self::request_for_window(cx.app, cx.window, request);
    }

    pub fn request_for_window<H: UiHost>(
        app: &mut H,
        window: AppWindowId,
        request: OverlayRequest,
    ) {
        match request.kind {
            OverlayKind::NonModalDismissible => {
                let open = request
                    .open
                    .expect("NonModalDismissible requires open model");
                let trigger = request
                    .trigger
                    .expect("NonModalDismissible requires trigger");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::popover_root_name(request.id));
                window_overlays::request_dismissible_popover_for_window(
                    app,
                    window,
                    window_overlays::DismissiblePopoverRequest::new(
                        request.id,
                        trigger,
                        open,
                        request.children,
                    )
                    .root_name(root_name)
                    .dismissable_branches(request.dismissable_branches)
                    .consume_outside_pointer_events(request.consume_outside_pointer_events)
                    .disable_outside_pointer_events(request.disable_outside_pointer_events)
                    .close_on_window_focus_lost(request.close_on_window_focus_lost)
                    .close_on_window_resize(request.close_on_window_resize)
                    .present(request.presence.present)
                    .initial_focus(request.initial_focus)
                    .on_open_auto_focus(request.on_open_auto_focus)
                    .on_close_auto_focus(request.on_close_auto_focus)
                    .on_dismiss_request(request.dismissible_on_dismiss_request)
                    .on_pointer_move(request.dismissible_on_pointer_move),
                );
            }
            OverlayKind::Modal => {
                let open = request.open.expect("Modal requires open model");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::modal_root_name(request.id));
                window_overlays::request_modal_for_window(
                    app,
                    window,
                    window_overlays::ModalRequest::new(request.id, open, request.children)
                        .root_name(root_name)
                        .trigger(request.trigger)
                        .close_on_window_focus_lost(request.close_on_window_focus_lost)
                        .close_on_window_resize(request.close_on_window_resize)
                        .present(request.presence.present)
                        .initial_focus(request.initial_focus)
                        .on_open_auto_focus(request.on_open_auto_focus)
                        .on_close_auto_focus(request.on_close_auto_focus)
                        .on_dismiss_request(request.dismissible_on_dismiss_request),
                );
            }
            OverlayKind::Tooltip => {
                if !request.presence.present {
                    return;
                }
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::tooltip_root_name(request.id));
                window_overlays::request_tooltip_for_window(
                    app,
                    window,
                    window_overlays::TooltipRequest::new(request.id, request.children)
                        .root_name(root_name)
                        .interactive(request.presence.interactive)
                        .trigger(request.trigger)
                        .on_dismiss_request(request.dismissible_on_dismiss_request)
                        .on_pointer_move(request.dismissible_on_pointer_move),
                );
            }
            OverlayKind::Hover => {
                let trigger = request.trigger.expect("Hover requires trigger");
                if !request.presence.present {
                    return;
                }
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::hover_overlay_root_name(request.id));
                window_overlays::request_hover_overlay_for_window(
                    app,
                    window,
                    window_overlays::HoverOverlayRequest::new(
                        request.id,
                        trigger,
                        request.children,
                    )
                    .root_name(root_name)
                    .interactive(request.presence.interactive),
                );
            }
            OverlayKind::ToastLayer => {
                let spec = request
                    .toast_layer
                    .expect("ToastLayer requires toast_layer spec");
                let root_name = request
                    .root_name
                    .unwrap_or_else(|| window_overlays::toast_layer_root_name(request.id));

                let mut toast_req = window_overlays::ToastLayerRequest::new(request.id, spec.store)
                    .position(spec.position)
                    .style(spec.style)
                    .root_name(root_name);
                if let Some(margin) = spec.margin {
                    toast_req = toast_req.margin(margin);
                }
                if let Some(gap) = spec.gap {
                    toast_req = toast_req.gap(gap);
                }
                if let Some(width) = spec.toast_min_width {
                    toast_req = toast_req.toast_min_width(width);
                }
                if let Some(width) = spec.toast_max_width {
                    toast_req = toast_req.toast_max_width(width);
                }
                window_overlays::request_toast_layer_for_window(app, window, toast_req);
            }
        }
    }

    pub fn render<H: UiHost>(
        ui: &mut UiTree<H>,
        app: &mut H,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
    ) {
        window_overlays::render(ui, app, services, window, bounds);
    }

    /// Computes a stable snapshot of overlay-related input arbitration state from the runtime
    /// layer stack.
    ///
    /// Recommended usage:
    /// - call after `OverlayController::render(...)` (so the layer stack reflects current overlay state),
    /// - use the snapshot to drive cross-system policies (e.g. docking/viewport suppression, diagnostics).
    pub fn arbitration_snapshot<H: UiHost>(ui: &UiTree<H>) -> OverlayArbitrationSnapshot {
        use fret_ui::tree::PointerOcclusion;

        let runtime = ui.input_arbitration_snapshot();
        let base_root = ui.base_root();
        let layers = ui.debug_layers_in_paint_order();

        let mut out = OverlayArbitrationSnapshot::default();
        out.has_any_overlays = layers
            .iter()
            .any(|l| l.visible && base_root.is_none_or(|base| l.root != base));
        out.modal_barrier_active = runtime.modal_barrier_root.is_some();
        out.pointer_capture_active = runtime.pointer_capture_active;

        if out.modal_barrier_active {
            out.pointer_occlusion = PointerOcclusion::None;
            return out;
        }

        out.pointer_occlusion = runtime.pointer_occlusion;
        out
    }

    /// Computes an ordered, window-scoped overlay stack snapshot by combining:
    ///
    /// - runtime layer order (`UiTree::debug_layers_in_paint_order`),
    /// - overlay manager state (`window_overlays`) to map layer IDs to overlay IDs/kinds.
    ///
    /// The intent is to give ecosystem integration points a stable "what overlays are currently
    /// active, and in what order" view without requiring them to depend on `window_overlays`
    /// internals.
    pub fn stack_snapshot_for_window<H: UiHost>(
        ui: &UiTree<H>,
        app: &mut H,
        window: AppWindowId,
    ) -> WindowOverlayStackSnapshot {
        use fret_ui::tree::PointerOcclusion;
        use std::collections::HashMap;

        let arbitration = Self::arbitration_snapshot(ui);
        let base_root = ui.base_root();
        let layers = ui.debug_layers_in_paint_order();

        let mut by_layer = HashMap::new();
        for entry in window_overlays::overlay_layer_entries_for_window(app, window) {
            let kind = match entry.kind {
                window_overlays::WindowOverlayLayerKind::Popover => OverlayStackEntryKind::Popover,
                window_overlays::WindowOverlayLayerKind::Modal => OverlayStackEntryKind::Modal,
                window_overlays::WindowOverlayLayerKind::Hover => OverlayStackEntryKind::Hover,
                window_overlays::WindowOverlayLayerKind::Tooltip => OverlayStackEntryKind::Tooltip,
                window_overlays::WindowOverlayLayerKind::ToastLayer => {
                    OverlayStackEntryKind::ToastLayer
                }
            };
            by_layer.insert(entry.layer, (kind, entry.id, entry.open));
        }

        let mut stack: Vec<WindowOverlayStackEntry> = Vec::with_capacity(layers.len());
        for layer in layers {
            let (kind, id, open) = if base_root == Some(layer.root) {
                (OverlayStackEntryKind::Base, None, false)
            } else if let Some((kind, id, open)) = by_layer.get(&layer.id).copied() {
                (kind, Some(id), open)
            } else {
                (OverlayStackEntryKind::Unknown, None, false)
            };
            stack.push(WindowOverlayStackEntry {
                kind,
                id,
                open,
                visible: layer.visible,
                blocks_underlay_input: layer.blocks_underlay_input,
                hit_testable: layer.hit_testable,
                pointer_occlusion: layer.pointer_occlusion,
            });
        }

        let topmost_overlay = stack
            .iter()
            .rev()
            .find_map(|e| (e.visible && e.id.is_some()).then_some(e.id))
            .flatten();
        let topmost_popover = stack
            .iter()
            .rev()
            .find_map(|e| {
                (e.visible && e.open && e.kind == OverlayStackEntryKind::Popover).then_some(e.id)
            })
            .flatten();
        let topmost_modal = stack
            .iter()
            .rev()
            .find_map(|e| (e.visible && e.kind == OverlayStackEntryKind::Modal).then_some(e.id))
            .flatten();
        let topmost_pointer_occluding_overlay = stack
            .iter()
            .rev()
            .find_map(|e| {
                (e.visible && e.pointer_occlusion != PointerOcclusion::None).then_some(e.id)
            })
            .flatten();

        WindowOverlayStackSnapshot {
            arbitration,
            stack,
            topmost_overlay,
            topmost_popover,
            topmost_modal,
            topmost_pointer_occluding_overlay,
        }
    }

    pub fn fade_presence<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        fade_ticks: u64,
    ) -> PresenceOutput {
        presence::fade_presence(cx, open, fade_ticks)
    }

    pub fn fade_presence_with_durations<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        open_ticks: u64,
        close_ticks: u64,
    ) -> PresenceOutput {
        presence::fade_presence_with_durations(cx, open, open_ticks, close_ticks)
    }

    /// Drive a general transition timeline using the UI runtime's monotonic frame clock.
    ///
    /// This is the generalized form of `fade_presence*` and is useful for driving multiple
    /// properties (opacity/scale/translation) with a shared open/close timeline.
    pub fn transition<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        ticks: u64,
    ) -> TransitionOutput {
        crate::declarative::transition::drive_transition(cx, open, ticks)
    }

    /// Drive a general transition timeline with separate open/close durations.
    pub fn transition_with_durations<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        open_ticks: u64,
        close_ticks: u64,
    ) -> TransitionOutput {
        crate::declarative::transition::drive_transition_with_durations(
            cx,
            open,
            open_ticks,
            close_ticks,
        )
    }

    /// Drive a transition timeline with an explicit easing function.
    ///
    /// This enables CSS-style easing (e.g. cubic-bezier) while staying deterministic and
    /// renderer-agnostic.
    pub fn transition_with_durations_and_easing<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        open_ticks: u64,
        close_ticks: u64,
        ease: fn(f32) -> f32,
    ) -> TransitionOutput {
        crate::declarative::transition::drive_transition_with_durations_and_easing(
            cx,
            open,
            open_ticks,
            close_ticks,
            ease,
        )
    }

    pub fn transition_with_durations_and_cubic_bezier<H: UiHost>(
        cx: &mut ElementContext<'_, H>,
        open: bool,
        open_ticks: u64,
        close_ticks: u64,
        bezier: fret_ui::theme::CubicBezier,
    ) -> TransitionOutput {
        crate::declarative::transition::drive_transition_with_durations_and_cubic_bezier(
            cx,
            open,
            open_ticks,
            close_ticks,
            bezier,
        )
    }

    pub fn toast_store<H: UiHost>(app: &mut H) -> Model<window_overlays::ToastStore> {
        window_overlays::toast_store(app)
    }

    pub fn toast_action(
        host: &mut dyn fret_ui::action::UiActionHost,
        store: Model<window_overlays::ToastStore>,
        window: AppWindowId,
        request: window_overlays::ToastRequest,
    ) -> window_overlays::ToastId {
        window_overlays::toast_action(host, store, window, request)
    }

    pub fn dismiss_toast_action(
        host: &mut dyn fret_ui::action::UiActionHost,
        store: Model<window_overlays::ToastStore>,
        window: AppWindowId,
        id: window_overlays::ToastId,
    ) -> bool {
        window_overlays::dismiss_toast_action(host, store, window, id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        Event, KeyCode, Modifiers, Point, Px, Rect, TextBlobId, TextConstraints, TextInput,
        TextMetrics, TextService,
    };
    use fret_core::{PathCommand, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_runtime::CommandId;
    use fret_runtime::Effect;
    use fret_ui::element::{LayoutStyle, Length, PointerRegionProps, PressableProps};
    use std::sync::Arc;

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

    #[test]
    fn arbitration_snapshot_reports_modal_and_pointer_occlusion() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |_| Vec::new(),
        );
        ui.set_root(base);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = OverlayController::arbitration_snapshot(&ui);
        assert_eq!(
            snap,
            OverlayArbitrationSnapshot {
                has_any_overlays: false,
                modal_barrier_active: false,
                pointer_occlusion: fret_ui::tree::PointerOcclusion::None,
                pointer_capture_active: false,
            }
        );

        // Add a non-modal overlay with pointer occlusion.
        OverlayController::begin_frame(&mut app, window);
        let overlay = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "overlay",
            |_| Vec::new(),
        );
        let overlay_layer = ui.push_overlay_root_ex(overlay, false, true);
        ui.set_layer_pointer_occlusion(
            overlay_layer,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = OverlayController::arbitration_snapshot(&ui);
        assert_eq!(snap.has_any_overlays, true);
        assert_eq!(snap.modal_barrier_active, false);
        assert_eq!(
            snap.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::BlockMouseExceptScroll
        );

        // Add a modal barrier; it should override occlusion in the snapshot.
        OverlayController::begin_frame(&mut app, window);
        let modal = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "modal",
            |_| Vec::new(),
        );
        let _modal_layer = ui.push_overlay_root_ex(modal, true, true);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = OverlayController::arbitration_snapshot(&ui);
        assert_eq!(snap.has_any_overlays, true);
        assert_eq!(snap.modal_barrier_active, true);
        assert_eq!(
            snap.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
    }

    #[test]
    fn arbitration_snapshot_reports_pointer_capture_active() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![cx.pointer_region(
                    PointerRegionProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Fill;
                            layout.size.height = Length::Fill;
                            layout
                        },
                        enabled: true,
                    },
                    |cx| {
                        cx.pointer_region_on_pointer_down(Arc::new(move |host, _cx, _down| {
                            host.capture_pointer();
                            true
                        }));
                        Vec::new()
                    },
                )]
            },
        );
        ui.set_root(base);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                click_count: 1,
                pointer_id: fret_core::PointerId(0),
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        let snap = OverlayController::arbitration_snapshot(&ui);
        assert_eq!(snap.has_any_overlays, false);
        assert_eq!(snap.modal_barrier_active, false);
        assert_eq!(
            snap.pointer_occlusion,
            fret_ui::tree::PointerOcclusion::None
        );
        assert_eq!(snap.pointer_capture_active, true);
    }

    #[test]
    fn stack_snapshot_reports_topmost_popover_and_modal_in_paint_order() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let popover_open = app.models_mut().insert(true);
        let modal_open = app.models_mut().insert(true);

        let mut services = FakeServices::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        let mut trigger_id: Option<GlobalElementId> = None;

        // Frame 0: base only.
        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![cx.pressable_with_id(
                    PressableProps {
                        layout: {
                            let mut layout = LayoutStyle::default();
                            layout.size.width = Length::Px(Px(80.0));
                            layout.size.height = Length::Px(Px(32.0));
                            layout
                        },
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        trigger_id = Some(id);
                        Vec::new()
                    },
                )]
            },
        );
        ui.set_root(base);
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let trigger_id = trigger_id.expect("trigger id");

        // Frame 1: popover opened.
        OverlayController::begin_frame(&mut app, window);
        let popover_id = GlobalElementId(0xabc);
        OverlayController::request_for_window(
            &mut app,
            window,
            OverlayRequest::dismissible_menu(
                popover_id,
                trigger_id,
                popover_open.clone(),
                OverlayPresence::instant(true),
                Vec::new(),
            ),
        );
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        assert_eq!(snap.topmost_popover, Some(popover_id));
        assert_eq!(snap.topmost_modal, None);
        assert_eq!(snap.topmost_overlay, Some(popover_id));
        assert_eq!(
            snap.stack.last().map(|e| (e.kind, e.id)),
            Some((OverlayStackEntryKind::Popover, Some(popover_id)))
        );

        // Frame 2: modal opened above the popover.
        OverlayController::begin_frame(&mut app, window);
        let modal_id = GlobalElementId(0xdef);
        OverlayController::request_for_window(
            &mut app,
            window,
            OverlayRequest::dismissible_menu(
                popover_id,
                trigger_id,
                popover_open.clone(),
                OverlayPresence::instant(true),
                Vec::new(),
            ),
        );
        OverlayController::request_for_window(
            &mut app,
            window,
            OverlayRequest::modal(
                modal_id,
                Some(trigger_id),
                modal_open.clone(),
                OverlayPresence::instant(true),
                Vec::new(),
            ),
        );
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = OverlayController::stack_snapshot_for_window(&ui, &mut app, window);
        assert_eq!(snap.topmost_modal, Some(modal_id));
        assert_eq!(snap.topmost_overlay, Some(modal_id));
        assert!(
            snap.stack
                .iter()
                .any(|e| e.kind == OverlayStackEntryKind::Popover && e.id == Some(popover_id)),
            "expected popover layer to still be identifiable in the stack snapshot even if it closes on modal open"
        );
        assert_eq!(
            snap.stack.last().map(|e| (e.kind, e.id)),
            Some((OverlayStackEntryKind::Modal, Some(modal_id)))
        );
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn dispatch_keydown_and_apply_commands(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        key: KeyCode,
        modifiers: Modifiers,
    ) {
        ui.dispatch_event(
            app,
            services,
            &Event::KeyDown {
                key,
                modifiers,
                repeat: false,
            },
        );

        for effect in app.flush_effects() {
            let Effect::Command { command, .. } = effect else {
                continue;
            };
            let _ = ui.dispatch_command(app, services, &command);
        }
    }

    #[test]
    fn toast_layer_request_enables_timer_events_when_toasts_exist() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        // Base root.
        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |_| Vec::new(),
        );
        ui.set_root(base);

        // Add a toast entry.
        let store = OverlayController::toast_store(&mut app);
        let _ = OverlayController::toast_action(
            &mut fret_ui::action::UiActionHostAdapter { app: &mut app },
            store.clone(),
            window,
            window_overlays::ToastRequest::new("Hello"),
        );

        // Request toast layer through the controller and render.
        OverlayController::begin_frame(&mut app, window);
        OverlayController::request_for_window(
            &mut app,
            window,
            OverlayRequest::toast_layer(GlobalElementId(0xbeef), store)
                .toast_position(window_overlays::ToastPosition::BottomRight),
        );
        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let layers = ui.debug_layers_in_paint_order();
        assert!(
            layers.iter().any(|l| l.wants_timer_events),
            "expected at least one layer to request timer events when toasts exist"
        );
    }

    #[test]
    fn modal_focus_traversal_is_scoped_to_modal_layer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        let mut underlay_a: Option<GlobalElementId> = None;
        let mut underlay_b: Option<GlobalElementId> = None;

        // Base root with two focusable pressables (underlay).
        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |cx| {
                vec![
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            underlay_a = Some(id);
                            Vec::new()
                        },
                    ),
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            underlay_b = Some(id);
                            Vec::new()
                        },
                    ),
                ]
            },
        );
        ui.set_root(base);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let underlay_a = underlay_a.expect("underlay a id");
        let underlay_b = underlay_b.expect("underlay b id");
        let underlay_a_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_a).expect("underlay a");
        let underlay_b_node =
            fret_ui::elements::node_for_element(&mut app, window, underlay_b).expect("underlay b");

        // Request a modal with two focusable pressables (modal layer).
        let open = app.models_mut().insert(true);
        let mut modal_a: Option<GlobalElementId> = None;
        let mut modal_b: Option<GlobalElementId> = None;

        OverlayController::begin_frame(&mut app, window);
        let modal_children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
                vec![
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            modal_a = Some(id);
                            Vec::new()
                        },
                    ),
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            modal_b = Some(id);
                            Vec::new()
                        },
                    ),
                ]
            });

        let modal_a = modal_a.expect("modal a id");
        let modal_b = modal_b.expect("modal b id");

        let mut req = OverlayRequest::modal(
            GlobalElementId(0x1234),
            None,
            open,
            OverlayPresence::instant(true),
            modal_children,
        );
        req.initial_focus = Some(modal_a);
        OverlayController::request_for_window(&mut app, window, req);

        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let modal_a_node =
            fret_ui::elements::node_for_element(&mut app, window, modal_a).expect("modal a");
        let modal_b_node =
            fret_ui::elements::node_for_element(&mut app, window, modal_b).expect("modal b");

        assert_eq!(ui.focus(), Some(modal_a_node));

        // Focus traversal must be scoped to the modal layer while the barrier is installed.
        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(modal_b_node));
        assert_ne!(ui.focus(), Some(underlay_a_node));
        assert_ne!(ui.focus(), Some(underlay_b_node));

        let _ = ui.dispatch_command(&mut app, &mut services, &CommandId::from("focus.next"));
        assert_eq!(ui.focus(), Some(modal_a_node));
        assert_ne!(ui.focus(), Some(underlay_a_node));
        assert_ne!(ui.focus(), Some(underlay_b_node));
    }

    #[test]
    fn modal_tab_keydown_cycles_focus_within_modal() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(300.0), Px(200.0)),
        );

        let open = app.models_mut().insert(true);
        let mut modal_a: Option<GlobalElementId> = None;
        let mut modal_b: Option<GlobalElementId> = None;

        // Base root is required so the window exists and input dispatch can proceed.
        OverlayController::begin_frame(&mut app, window);
        let base = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "base",
            |_| Vec::new(),
        );
        ui.set_root(base);

        // Request a modal with two focusables. We'll drive Tab/Shift+Tab via KeyDown events.
        OverlayController::begin_frame(&mut app, window);
        let modal_children =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "modal-child", |cx| {
                vec![
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            modal_a = Some(id);
                            Vec::new()
                        },
                    ),
                    cx.pressable_with_id(
                        PressableProps {
                            layout: {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Px(Px(80.0));
                                layout.size.height = Length::Px(Px(32.0));
                                layout
                            },
                            focusable: true,
                            ..Default::default()
                        },
                        |_cx, _st, id| {
                            modal_b = Some(id);
                            Vec::new()
                        },
                    ),
                ]
            });

        let modal_a = modal_a.expect("modal a id");
        let modal_b = modal_b.expect("modal b id");

        let mut req = OverlayRequest::modal(
            GlobalElementId(0x1234),
            None,
            open,
            OverlayPresence::instant(true),
            modal_children,
        );
        req.initial_focus = Some(modal_a);
        OverlayController::request_for_window(&mut app, window, req);

        OverlayController::render(&mut ui, &mut app, &mut services, window, bounds);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let modal_a_node =
            fret_ui::elements::node_for_element(&mut app, window, modal_a).expect("modal a");
        let modal_b_node =
            fret_ui::elements::node_for_element(&mut app, window, modal_b).expect("modal b");

        assert_eq!(ui.focus(), Some(modal_a_node));

        // Tab => focus.next
        dispatch_keydown_and_apply_commands(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::Tab,
            Modifiers::default(),
        );
        assert_eq!(ui.focus(), Some(modal_b_node));

        // Tab => wraps within modal
        dispatch_keydown_and_apply_commands(
            &mut ui,
            &mut app,
            &mut services,
            KeyCode::Tab,
            Modifiers::default(),
        );
        assert_eq!(ui.focus(), Some(modal_a_node));

        // Shift+Tab => focus.previous
        let mods = Modifiers {
            shift: true,
            ..Default::default()
        };
        dispatch_keydown_and_apply_commands(&mut ui, &mut app, &mut services, KeyCode::Tab, mods);
        assert_eq!(ui.focus(), Some(modal_b_node));
    }
}
