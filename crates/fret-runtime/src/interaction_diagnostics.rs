use std::collections::HashMap;

use fret_core::geometry::{Point, Rect};
use fret_core::{AppWindowId, Axis, DockNodeId, DropZone, PointerId, RenderTargetId};

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
    /// Best-effort dock graph stats snapshot for the current window.
    pub dock_graph_stats: Option<DockGraphStatsDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockGraphStatsDiagnostics {
    pub node_count: u32,
    pub tabs_count: u32,
    pub split_count: u32,
    pub floating_count: u32,
    pub max_depth: u32,
    pub max_split_depth: u32,
    /// True when the graph satisfies the key canonical-form invariants used by docking.
    pub canonical_ok: bool,
    /// True when a split contains a same-axis split child (an indicator of unflattened nesting).
    pub has_nested_same_axis_splits: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDropPreviewKindDiagnostics {
    WrapBinary,
    InsertIntoSplit {
        axis: Axis,
        split: DockNodeId,
        insert_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDropPreviewDiagnostics {
    pub kind: DockDropPreviewKindDiagnostics,
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
    pub preview: Option<DockDropPreviewDiagnostics>,
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
    latest_docking: DockingInteractionDiagnostics,
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
        w.docking = diagnostics.clone();
        w.latest_docking = diagnostics;
    }

    pub fn docking_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&DockingInteractionDiagnostics> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(&w.docking)
    }

    pub fn docking_latest_for_window(
        &self,
        window: AppWindowId,
    ) -> Option<&DockingInteractionDiagnostics> {
        self.per_window.get(&window).map(|w| &w.latest_docking)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn docking_latest_is_stable_across_begin_frame_resets() {
        let mut store = WindowInteractionDiagnosticsStore::default();
        let window = AppWindowId::default();

        let snapshot = DockingInteractionDiagnostics {
            dock_graph_stats: Some(DockGraphStatsDiagnostics {
                node_count: 3,
                tabs_count: 1,
                split_count: 1,
                floating_count: 0,
                max_depth: 2,
                max_split_depth: 1,
                canonical_ok: true,
                has_nested_same_axis_splits: false,
            }),
            ..Default::default()
        };

        store.record_docking(window, FrameId(1), snapshot);
        store.begin_frame(window, FrameId(2));

        assert!(
            store
                .docking_latest_for_window(window)
                .and_then(|d| d.dock_graph_stats)
                .is_some_and(|s| s.canonical_ok),
            "latest snapshot should persist even when the current frame snapshot is reset"
        );

        assert!(
            store
                .docking_for_window(window, FrameId(2))
                .is_some_and(|d| d.dock_graph_stats.is_none()),
            "frame-scoped snapshot should be cleared by begin_frame when not recorded"
        );
    }
}
