//! Window-scoped overlay manager (policy layer).
//!
//! This is a small component-layer orchestration helper that installs `UiTree` overlay roots
//! (ADR 0067) and coordinates dismissal + focus restore rules (ADR 0069).

use std::collections::HashMap;

mod frame;
mod names;
mod render;
mod requests;
mod state;
mod toast;

#[cfg(test)]
mod tests;

use fret_core::AppWindowId;
use fret_runtime::FrameId;
use fret_ui::elements::GlobalElementId;
use fret_ui::tree::UiLayerId;
use fret_ui::{Invalidation, UiHost, UiTree};

pub use frame::{
    begin_frame, request_dismissible_popover_for_window, request_hover_overlay_for_window,
    request_modal_for_window, request_toast_layer_for_window, request_tooltip_for_window,
};

#[cfg(feature = "unstable-internals")]
pub use frame::{
    request_dismissible_popover, request_hover_overlay, request_modal, request_toast_layer,
    request_tooltip,
};
pub use names::{
    hover_overlay_root_name, modal_root_name, popover_root_name, toast_layer_root_name,
    tooltip_root_name,
};
pub use render::render;
pub use requests::{
    DismissiblePopoverRequest, HoverOverlayRequest, ModalRequest, ToastButtonStyle,
    ToastIconButtonStyle, ToastLayerRequest, ToastLayerStyle, ToastTextStyle, ToastVariantColors,
    ToastVariantPalette, TooltipRequest,
};
pub use toast::{
    DEFAULT_MAX_TOASTS, DEFAULT_SWIPE_DRAGGING_THRESHOLD_PX, DEFAULT_SWIPE_MAX_DRAG_PX,
    DEFAULT_SWIPE_THRESHOLD_PX, ToastAction, ToastId, ToastPosition, ToastRequest, ToastStore,
    ToastSwipeConfig, ToastSwipeDirection, ToastVariant, dismiss_toast_action, toast_action,
    toast_store,
};

/// Radix `ToastViewport` focus-jump command (default hotkey: `F8`).
pub const TOAST_VIEWPORT_FOCUS_COMMAND: &str = "toast.viewport.focus";

/// Attempts to handle a window-scoped command that targets overlay substrates.
///
/// This lives in `fret-ui-kit` (not `fret-ui`) because it needs access to overlay controller
/// state (e.g. active toast layers).
pub fn try_handle_window_command<H: UiHost>(
    ui: &mut UiTree<H>,
    app: &mut H,
    window: AppWindowId,
    command: &fret_runtime::CommandId,
) -> bool {
    if command.as_str() != TOAST_VIEWPORT_FOCUS_COMMAND {
        return false;
    }

    let layer = app.with_global_mut_untracked(state::WindowOverlays::default, |overlays, _app| {
        overlays
            .toast_layers
            .iter()
            .find_map(|((w, _id), active)| (*w == window).then_some(active.layer))
    });

    let Some(layer) = layer else {
        return false;
    };
    if !ui.is_layer_visible(layer) {
        return false;
    }

    let Some(root) = ui.layer_root(layer) else {
        return false;
    };

    if let Some(prev) = ui.focus() {
        ui.invalidate_with_source(
            prev,
            Invalidation::Paint,
            fret_ui::tree::UiDebugInvalidationSource::Focus,
        );
    }
    ui.set_focus(Some(root));
    ui.invalidate_with_source(
        root,
        Invalidation::Paint,
        fret_ui::tree::UiDebugInvalidationSource::Focus,
    );
    app.request_redraw(window);
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WindowOverlayLayerKind {
    Popover,
    Modal,
    Hover,
    Tooltip,
    ToastLayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WindowOverlayLayerEntry {
    pub kind: WindowOverlayLayerKind,
    pub id: GlobalElementId,
    pub layer: UiLayerId,
    pub open: bool,
}

pub(crate) fn overlay_layer_entries_for_window<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
) -> Vec<WindowOverlayLayerEntry> {
    app.with_global_mut_untracked(state::WindowOverlays::default, |overlays, app| {
        let mut out: Vec<WindowOverlayLayerEntry> = Vec::new();

        for ((w, id), active) in overlays.popovers.iter() {
            if *w != window {
                continue;
            }
            out.push(WindowOverlayLayerEntry {
                kind: WindowOverlayLayerKind::Popover,
                id: *id,
                layer: active.layer,
                open: active.open,
            });
        }

        for ((w, id), active) in overlays.modals.iter() {
            if *w != window {
                continue;
            }
            out.push(WindowOverlayLayerEntry {
                kind: WindowOverlayLayerKind::Modal,
                id: *id,
                layer: active.layer,
                open: active.open,
            });
        }

        for ((w, id), active) in overlays.hover_overlays.iter() {
            if *w != window {
                continue;
            }
            let open = app.models().get_copied(&active.open).unwrap_or(false);
            out.push(WindowOverlayLayerEntry {
                kind: WindowOverlayLayerKind::Hover,
                id: *id,
                layer: active.layer,
                open,
            });
        }

        for ((w, id), active) in overlays.tooltips.iter() {
            if *w != window {
                continue;
            }
            let open = app.models().get_copied(&active.open).unwrap_or(false);
            out.push(WindowOverlayLayerEntry {
                kind: WindowOverlayLayerKind::Tooltip,
                id: *id,
                layer: active.layer,
                open,
            });
        }

        for ((w, id), active) in overlays.toast_layers.iter() {
            if *w != window {
                continue;
            }
            out.push(WindowOverlayLayerEntry {
                kind: WindowOverlayLayerKind::ToastLayer,
                id: *id,
                layer: active.layer,
                open: true,
            });
        }

        out
    })
}

/// Tracks which window overlays were synthesized from cached request declarations.
///
/// This is intended for diagnostics and scripted regressions: when view caching skips rerendering
/// overlay producers, cached synthesis keeps behavior stable. Recording this makes it possible to
/// assert that synthesis happened (or understand why it didn't) from exported `bundle.json` files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySynthesisKind {
    Modal,
    Popover,
    Hover,
    Tooltip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySynthesisSource {
    CachedDeclaration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlaySynthesisOutcome {
    Synthesized,
    SuppressedMissingTrigger,
    SuppressedTriggerNotLiveInCurrentFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlaySynthesisEvent {
    pub kind: OverlaySynthesisKind,
    pub id: GlobalElementId,
    pub source: OverlaySynthesisSource,
    pub outcome: OverlaySynthesisOutcome,
}

#[derive(Default)]
pub struct WindowOverlaySynthesisDiagnosticsStore {
    per_window: HashMap<AppWindowId, WindowOverlaySynthesisDiagnosticsFrame>,
}

#[derive(Default)]
struct WindowOverlaySynthesisDiagnosticsFrame {
    frame_id: FrameId,
    events: Vec<OverlaySynthesisEvent>,
}

impl WindowOverlaySynthesisDiagnosticsStore {
    pub fn begin_frame(&mut self, window: AppWindowId, frame_id: FrameId) {
        let w = self.per_window.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.events.clear();
        }
    }

    pub fn record_events(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        events: impl IntoIterator<Item = OverlaySynthesisEvent>,
    ) {
        self.begin_frame(window, frame_id);
        let w = self.per_window.entry(window).or_default();
        w.events.extend(events);
    }

    #[allow(dead_code)]
    pub fn events_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&[OverlaySynthesisEvent]> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(w.events.as_slice())
    }
}
