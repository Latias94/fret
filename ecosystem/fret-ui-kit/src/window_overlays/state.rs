use std::collections::HashMap;

use fret_core::{AppWindowId, NodeId, Rect};
use fret_runtime::FrameId;
use fret_ui::action::{OnCloseAutoFocus, OnOpenAutoFocus};
use fret_ui::tree::UiLayerId;
use fret_ui::{UiHost, UiTree};

use super::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, TooltipRequest,
};
use fret_ui::elements::GlobalElementId;

#[derive(Default)]
pub(super) struct WindowOverlayFrame {
    pub(super) frame_id: FrameId,
    pub(super) last_bounds: Option<Rect>,
    pub(super) last_focused: Option<bool>,
    pub(super) last_scale_factor: Option<f32>,
    pub(super) dock_drag_active_last: bool,
    pub(super) dock_drag_restore_focus: Option<NodeId>,
    pub(super) popovers: Vec<DismissiblePopoverRequest>,
    pub(super) modals: Vec<ModalRequest>,
    pub(super) hover_overlays: Vec<HoverOverlayRequest>,
    pub(super) tooltips: Vec<TooltipRequest>,
    pub(super) toasts: Vec<ToastLayerRequest>,
}

pub(super) struct ActivePopover {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
    pub(super) trigger: GlobalElementId,
    pub(super) initial_focus: Option<GlobalElementId>,
    pub(super) on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub(super) on_close_auto_focus: Option<OnCloseAutoFocus>,
    pub(super) consume_outside_pointer_events: bool,
    pub(super) disable_outside_pointer_events: bool,
    pub(super) open: bool,
    pub(super) restore_focus: Option<NodeId>,
    pub(super) last_focus: Option<NodeId>,
}

pub(super) struct ActiveModal {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) initial_focus: Option<GlobalElementId>,
    pub(super) on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub(super) on_close_auto_focus: Option<OnCloseAutoFocus>,
    pub(super) open: bool,
    pub(super) restore_focus: Option<NodeId>,
    pub(super) pending_initial_focus: bool,
}

pub(super) struct ActiveTooltip {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
}

pub(super) struct ActiveToastLayer {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
}

pub(super) struct ActiveHoverOverlay {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
    pub(super) trigger: GlobalElementId,
}

#[derive(Default)]
pub(super) struct WindowOverlays {
    pub(super) windows: HashMap<AppWindowId, WindowOverlayFrame>,
    /// Last-known request declarations for a given window/id.
    ///
    /// These are persisted across frames so per-frame request lists can be treated as an
    /// optimization rather than a hard requirement. This is critical when view caching skips
    /// rerendering the subtree that normally emits overlay requests.
    pub(super) cached_popover_requests:
        HashMap<(AppWindowId, GlobalElementId), DismissiblePopoverRequest>,
    /// See `cached_popover_requests`.
    pub(super) cached_modal_requests: HashMap<(AppWindowId, GlobalElementId), ModalRequest>,
    /// See `cached_popover_requests`.
    pub(super) cached_toast_layer_requests:
        HashMap<(AppWindowId, GlobalElementId), ToastLayerRequest>,
    pub(super) popovers: HashMap<(AppWindowId, GlobalElementId), ActivePopover>,
    pub(super) modals: HashMap<(AppWindowId, GlobalElementId), ActiveModal>,
    pub(super) hover_overlays: HashMap<(AppWindowId, GlobalElementId), ActiveHoverOverlay>,
    pub(super) tooltips: HashMap<(AppWindowId, GlobalElementId), ActiveTooltip>,
    pub(super) toast_layers: HashMap<(AppWindowId, GlobalElementId), ActiveToastLayer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverlayLayerKind {
    Modal,
    NonModalDismissible,
    Toast,
}

#[derive(Debug, Clone, Copy)]
struct OverlayLayerState {
    /// Whether the layer should be visible/painted.
    present: bool,
    /// Whether the overlay content should be interactive.
    ///
    /// For non-modal overlays this controls hit-testing and outside-press dismissal participation.
    /// For modal overlays, barrier semantics are driven by `present` (see `OverlayLayerKind::Modal`).
    interactive: bool,
    /// Whether this layer wants timer events (e.g. toast expiration).
    wants_timer_events: bool,
}

impl OverlayLayerState {
    fn hidden() -> Self {
        Self {
            present: false,
            interactive: false,
            wants_timer_events: false,
        }
    }

    fn modal(present: bool, interactive: bool) -> Self {
        Self {
            present,
            interactive,
            wants_timer_events: false,
        }
    }

    fn non_modal_dismissible(present: bool, interactive: bool) -> Self {
        Self {
            present,
            interactive,
            // Non-modal overlays may rely on timers for small interaction policies (e.g. submenu
            // safe-hover close delays). These policies should only run while the overlay is
            // interactive. During close transitions (`present=true` but `interactive=false`), the
            // layer must not participate in timer-driven interaction state machines.
            wants_timer_events: present && interactive,
        }
    }
    fn toast(present: bool, wants_timer_events: bool) -> Self {
        Self {
            present,
            interactive: present,
            wants_timer_events,
        }
    }
}

fn apply_overlay_layer_state<H: UiHost>(
    ui: &mut UiTree<H>,
    layer: UiLayerId,
    kind: OverlayLayerKind,
    st: OverlayLayerState,
) {
    ui.set_layer_wants_timer_events(layer, st.wants_timer_events);

    match kind {
        OverlayLayerKind::NonModalDismissible => {
            ui.set_layer_visible(layer, st.present);
            ui.set_layer_hit_testable(layer, st.interactive);
            ui.set_layer_wants_pointer_down_outside_events(layer, st.interactive);
            ui.set_layer_wants_pointer_move_events(layer, st.interactive);
        }
        OverlayLayerKind::Modal => {
            ui.set_layer_visible(layer, st.present);
            // For modal overlays, `present` is the authority for input gating. Even when a modal
            // is closing (`interactive=false` but `present=true` for an exit transition), the
            // layer must remain hit-testable to keep the underlay inert and prevent click-through.
            ui.set_layer_hit_testable(layer, st.present);
            ui.set_layer_wants_pointer_down_outside_events(layer, false);
        }
        OverlayLayerKind::Toast => {
            ui.set_layer_visible(layer, st.present);
            ui.set_layer_hit_testable(layer, st.interactive);
            ui.set_layer_wants_pointer_down_outside_events(layer, false);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct OverlayLayer {
    kind: OverlayLayerKind,
    state: OverlayLayerState,
}

impl OverlayLayer {
    fn new(kind: OverlayLayerKind, state: OverlayLayerState) -> Self {
        Self { kind, state }
    }

    fn hidden(kind: OverlayLayerKind) -> Self {
        Self::new(kind, OverlayLayerState::hidden())
    }

    pub(super) fn hide_modal() -> Self {
        Self::hidden(OverlayLayerKind::Modal)
    }

    pub(super) fn hide_non_modal_dismissible() -> Self {
        Self::hidden(OverlayLayerKind::NonModalDismissible)
    }

    pub(super) fn hide_toast() -> Self {
        Self::hidden(OverlayLayerKind::Toast)
    }

    pub(super) fn modal(present: bool, interactive: bool) -> Self {
        Self::new(
            OverlayLayerKind::Modal,
            OverlayLayerState::modal(present, interactive),
        )
    }

    pub(super) fn non_modal_dismissible(present: bool, interactive: bool) -> Self {
        Self::new(
            OverlayLayerKind::NonModalDismissible,
            OverlayLayerState::non_modal_dismissible(present, interactive),
        )
    }

    pub(super) fn toast(present: bool, wants_timer_events: bool) -> Self {
        Self::new(
            OverlayLayerKind::Toast,
            OverlayLayerState::toast(present, wants_timer_events),
        )
    }

    pub(super) fn apply<H: UiHost>(self, ui: &mut UiTree<H>, layer: UiLayerId) {
        apply_overlay_layer_state(ui, layer, self.kind, self.state);
    }
}
