use std::collections::HashMap;

use fret_core::geometry::{Point, Rect};
use fret_core::{AppWindowId, DockNodeId, DropZone, PointerId, RenderTargetId};

use crate::FrameId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDragDiagnostics {
    pub pointer_id: PointerId,
    pub source_window: AppWindowId,
    pub current_window: AppWindowId,
    pub dragging: bool,
    pub cross_window_hover: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportCaptureDiagnostics {
    pub pointer_id: PointerId,
    pub target: RenderTargetId,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DockingInteractionDiagnostics {
    pub dock_drag: Option<DockDragDiagnostics>,
    pub dock_drop_resolve: Option<DockDropResolveDiagnostics>,
    pub viewport_capture: Option<ViewportCaptureDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDropResolveSource {
    /// Docking previews are disabled for this drag session (inversion policy / modifier gating).
    InvertDocking,
    /// The cursor is outside the window bounds.
    OutsideWindow,
    /// The cursor is inside `float_zone(...)`, forcing in-window floating.
    FloatZone,
    /// The position is inside the window, but outside the computed docking layout bounds.
    LayoutBoundsMiss,
    /// The previous hover target was reused (anti-flicker latch).
    LatchedPreviousHover,
    /// The cursor hit the explicit tab-bar target (center docking + insert index).
    TabBar,
    /// The cursor is hovering an in-window floating container title bar (explicit target band).
    FloatingTitleBar,
    /// The cursor hit the outer direction-pad (window-root edge docking).
    OuterHintRect,
    /// The cursor hit the inner direction-pad (leaf docking).
    InnerHintRect,
    /// No docking drop target matched (gated by explicit-target rules).
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDropCandidateRectKind {
    WindowBounds,
    DockBounds,
    FloatZone,
    LayoutBounds,
    RootRect,
    LeafTabsRect,
    TabBarRect,
    InnerHintRect,
    OuterHintRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockDropCandidateRectDiagnostics {
    pub kind: DockDropCandidateRectKind,
    pub zone: Option<DropZone>,
    pub rect: Rect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDropTargetDiagnostics {
    pub layout_root: DockNodeId,
    pub tabs: DockNodeId,
    pub zone: DropZone,
    pub insert_index: Option<usize>,
    pub outer: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DockDropResolveDiagnostics {
    pub pointer_id: PointerId,
    pub position: Point,
    pub window_bounds: Rect,
    pub dock_bounds: Rect,
    pub source: DockDropResolveSource,
    pub resolved: Option<DockDropTargetDiagnostics>,
    pub candidates: Vec<DockDropCandidateRectDiagnostics>,
}

#[derive(Default)]
pub struct WindowInteractionDiagnosticsStore {
    per_window: HashMap<AppWindowId, WindowInteractionDiagnosticsFrame>,
}

#[derive(Default)]
struct WindowInteractionDiagnosticsFrame {
    frame_id: FrameId,
    docking: DockingInteractionDiagnostics,
}

impl WindowInteractionDiagnosticsStore {
    pub fn begin_frame(&mut self, window: AppWindowId, frame_id: FrameId) {
        let w = self.per_window.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.docking = DockingInteractionDiagnostics::default();
        }
    }

    pub fn record_docking(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        diagnostics: DockingInteractionDiagnostics,
    ) {
        self.begin_frame(window, frame_id);
        let w = self.per_window.entry(window).or_default();
        w.docking = diagnostics;
    }

    pub fn docking_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&DockingInteractionDiagnostics> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(&w.docking)
    }
}
