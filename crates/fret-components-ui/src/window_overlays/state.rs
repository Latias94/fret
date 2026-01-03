use std::collections::HashMap;

use fret_core::{AppWindowId, NodeId};
use fret_runtime::FrameId;
use fret_ui::tree::UiLayerId;
use fret_ui::{UiHost, UiTree};

use super::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastLayerRequest, TooltipRequest,
};
use fret_ui::elements::GlobalElementId;

#[derive(Default)]
pub(super) struct WindowOverlayFrame {
    pub(super) frame_id: FrameId,
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
    pub(super) open: bool,
    pub(super) restore_focus: Option<NodeId>,
    pub(super) last_focus: Option<NodeId>,
}

pub(super) struct ActiveModal {
    pub(super) layer: UiLayerId,
    pub(super) root_name: String,
    pub(super) trigger: Option<GlobalElementId>,
    pub(super) initial_focus: Option<GlobalElementId>,
    pub(super) open: bool,
    pub(super) restore_focus: Option<NodeId>,
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
    Hover,
    Tooltip,
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
            // safe-hover close delays). Keeping this enabled avoids requiring per-overlay opt-in
            // plumbing while the overlay policy surface is still evolving.
            wants_timer_events: present,
        }
    }

    fn tooltip(present: bool) -> Self {
        Self {
            present,
            interactive: false,
            wants_timer_events: false,
        }
    }

    fn hover(present: bool) -> Self {
        Self {
            present,
            interactive: present,
            wants_timer_events: false,
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
            // Modal barrier semantics are authoritative while the layer is present:
            // the barrier must keep blocking underlay input even when the modal is closing.
            ui.set_layer_hit_testable(layer, st.present);
            ui.set_layer_wants_pointer_down_outside_events(layer, false);
        }
        OverlayLayerKind::Tooltip => {
            ui.set_layer_visible(layer, st.present);
            ui.set_layer_hit_testable(layer, false);
            ui.set_layer_wants_pointer_down_outside_events(layer, false);
        }
        OverlayLayerKind::Hover => {
            ui.set_layer_visible(layer, st.present);
            ui.set_layer_hit_testable(layer, st.interactive);
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

    pub(super) fn hide_hover() -> Self {
        Self::hidden(OverlayLayerKind::Hover)
    }

    pub(super) fn hide_tooltip() -> Self {
        Self::hidden(OverlayLayerKind::Tooltip)
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

    pub(super) fn tooltip(present: bool) -> Self {
        Self::new(
            OverlayLayerKind::Tooltip,
            OverlayLayerState::tooltip(present),
        )
    }

    pub(super) fn hover(present: bool) -> Self {
        Self::new(OverlayLayerKind::Hover, OverlayLayerState::hover(present))
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
