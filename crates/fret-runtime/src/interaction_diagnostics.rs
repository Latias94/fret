use std::collections::HashMap;

use fret_core::geometry::{Point, Rect};
use fret_core::{AppWindowId, Axis, DockNodeId, DropZone, PointerId, RenderTargetId};

use crate::DragKindId;
use crate::FrameId;
use crate::WindowUnderCursorSource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDragDiagnostics {
    pub pointer_id: PointerId,
    pub source_window: AppWindowId,
    pub current_window: AppWindowId,
    /// The drag kind for the active dock drag session.
    pub kind: DragKindId,
    pub dragging: bool,
    pub cross_window_hover: bool,
    /// True when the runner has applied an ImGui-style "transparent payload" treatment to the
    /// moving dock window (e.g. click-through/NoInputs while following the cursor).
    pub transparent_payload_applied: bool,
    /// Best-effort diagnostics hint: true when the runner successfully applied click-through
    /// hit-test passthrough to the moving dock window while transparent payload is enabled.
    pub transparent_payload_hit_test_passthrough_applied: bool,
    /// Best-effort diagnostics hint: which mechanism was used to select the hovered window during
    /// cross-window drag routing (OS-backed vs heuristic).
    pub window_under_cursor_source: WindowUnderCursorSource,
    /// Best-effort diagnostics hint: OS window currently being moved by the runner for this drag
    /// session (ImGui-style "follow window" multi-viewport behavior).
    pub moving_window: Option<AppWindowId>,
    /// Best-effort diagnostics hint: when [`Self::moving_window`] is set, the window considered
    /// "under" the moving window at the current cursor position.
    pub window_under_moving_window: Option<AppWindowId>,
    /// Best-effort diagnostics hint: which mechanism was used to select
    /// [`Self::window_under_moving_window`].
    pub window_under_moving_window_source: WindowUnderCursorSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockFloatingDragDiagnostics {
    pub pointer_id: PointerId,
    pub floating: DockNodeId,
    pub activated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportCaptureDiagnostics {
    pub pointer_id: PointerId,
    pub target: RenderTargetId,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DockTabStripActiveVisibilityStatusDiagnostics {
    Ok,
    MissingWindowRoot,
    NoTabsFound,
    MissingLayoutRect,
    MissingTabsNode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DockTabStripActiveVisibilityDiagnostics {
    pub status: DockTabStripActiveVisibilityStatusDiagnostics,
    pub tabs_node: Option<DockNodeId>,
    /// True when the tab strip reports overflow (i.e. `max_scroll > 0`).
    pub overflow: bool,
    pub tab_count: usize,
    pub active: usize,
    pub scroll: fret_core::geometry::Px,
    pub max_scroll: fret_core::geometry::Px,
    /// True when `active` is visible at the current `scroll` (best-effort).
    pub active_visible: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DockingInteractionDiagnostics {
    pub dock_drag: Option<DockDragDiagnostics>,
    pub floating_drag: Option<DockFloatingDragDiagnostics>,
    pub dock_drop_resolve: Option<DockDropResolveDiagnostics>,
    pub viewport_capture: Option<ViewportCaptureDiagnostics>,
    /// Best-effort diagnostics for ensuring the active tab remains visible after selection.
    pub tab_strip_active_visibility: Option<DockTabStripActiveVisibilityDiagnostics>,
    /// Best-effort dock graph stats snapshot for the current window.
    pub dock_graph_stats: Option<DockGraphStatsDiagnostics>,
    /// Best-effort stable signature for the current window's dock graph.
    ///
    /// This is intended for scripted regression gates that want to assert an exact layout shape
    /// (dockview-style) without relying on pixels.
    pub dock_graph_signature: Option<DockGraphSignatureDiagnostics>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WorkspaceTabStripActiveVisibilityStatusDiagnostics {
    Ok,
    NoActiveTab,
    MissingScrollViewportRect,
    MissingActiveTabRect,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceTabStripActiveVisibilityDiagnostics {
    pub status: WorkspaceTabStripActiveVisibilityStatusDiagnostics,
    pub pane_id: Option<std::sync::Arc<str>>,
    pub active_tab_id: Option<std::sync::Arc<str>>,
    pub tab_count: usize,
    pub overflow: bool,
    pub scroll_x: fret_core::geometry::Px,
    pub max_scroll_x: fret_core::geometry::Px,
    pub scroll_viewport_rect: Option<Rect>,
    pub active_tab_rect: Option<Rect>,
    pub active_visible: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceTabStripDragDiagnostics {
    pub pane_id: Option<std::sync::Arc<str>>,
    pub pointer_id: Option<PointerId>,
    pub dragging: bool,
    pub dragged_tab_id: Option<std::sync::Arc<str>>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkspaceInteractionDiagnostics {
    /// Best-effort tab strip visibility diagnostics published by workspace shells.
    ///
    /// Multiple strips may exist per window (multi-pane); publishers should include `pane_id`
    /// so scripted gates can select deterministically.
    pub tab_strip_active_visibility: Vec<WorkspaceTabStripActiveVisibilityDiagnostics>,
    /// Best-effort drag state published by workspace shells.
    ///
    /// This is intended for scripted regression gates that want to assert "close buttons do not
    /// start drags" without relying on pixels.
    pub tab_strip_drag: Vec<WorkspaceTabStripDragDiagnostics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DockGraphSignatureDiagnostics {
    /// Stable, canonical-ish shape signature for the dock graph in a specific window.
    ///
    /// Notes:
    /// - Does not include floating window rects (platform-dependent).
    /// - Does not include split fractions (pointer-driven and DPI-sensitive).
    pub signature: String,
    /// FNV-1a 64-bit hash of `signature` (for compact assertions).
    pub fingerprint64: u64,
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
    /// The window has no dock root and the cursor is inside the dock bounds.
    ///
    /// Dropping in this state will create the initial root tab stack for the window.
    EmptyDockSpace,
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
    workspace: WorkspaceInteractionDiagnostics,
    latest_workspace: WorkspaceInteractionDiagnostics,
}

impl WindowInteractionDiagnosticsStore {
    pub fn begin_frame(&mut self, window: AppWindowId, frame_id: FrameId) {
        let w = self.per_window.entry(window).or_default();
        if w.frame_id != frame_id {
            w.frame_id = frame_id;
            w.docking = DockingInteractionDiagnostics::default();
            w.workspace = WorkspaceInteractionDiagnostics::default();
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

    pub fn record_workspace_tab_strip_active_visibility(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        diagnostics: WorkspaceTabStripActiveVisibilityDiagnostics,
    ) {
        self.begin_frame(window, frame_id);
        let w = self.per_window.entry(window).or_default();
        w.workspace.tab_strip_active_visibility.push(diagnostics);
        w.latest_workspace = w.workspace.clone();
    }

    pub fn record_workspace_tab_strip_drag(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        diagnostics: WorkspaceTabStripDragDiagnostics,
    ) {
        self.begin_frame(window, frame_id);
        let w = self.per_window.entry(window).or_default();
        w.workspace.tab_strip_drag.push(diagnostics);
        w.latest_workspace = w.workspace.clone();
    }

    pub fn docking_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&DockingInteractionDiagnostics> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(&w.docking)
    }

    pub fn workspace_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&WorkspaceInteractionDiagnostics> {
        let w = self.per_window.get(&window)?;
        (w.frame_id == frame_id).then_some(&w.workspace)
    }

    pub fn docking_latest_for_window(
        &self,
        window: AppWindowId,
    ) -> Option<&DockingInteractionDiagnostics> {
        self.per_window.get(&window).map(|w| &w.latest_docking)
    }

    pub fn workspace_latest_for_window(
        &self,
        window: AppWindowId,
    ) -> Option<&WorkspaceInteractionDiagnostics> {
        self.per_window.get(&window).map(|w| &w.latest_workspace)
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

    #[test]
    fn workspace_latest_is_stable_across_begin_frame_resets() {
        let mut store = WindowInteractionDiagnosticsStore::default();
        let window = AppWindowId::default();

        let snapshot = WorkspaceTabStripActiveVisibilityDiagnostics {
            status: WorkspaceTabStripActiveVisibilityStatusDiagnostics::Ok,
            pane_id: Some(std::sync::Arc::<str>::from("pane-a")),
            active_tab_id: Some(std::sync::Arc::<str>::from("doc-a-2")),
            tab_count: 3,
            overflow: true,
            scroll_x: fret_core::geometry::Px(12.0),
            max_scroll_x: fret_core::geometry::Px(120.0),
            scroll_viewport_rect: None,
            active_tab_rect: None,
            active_visible: true,
        };

        store.record_workspace_tab_strip_active_visibility(window, FrameId(1), snapshot);
        store.begin_frame(window, FrameId(2));

        assert!(
            store
                .workspace_latest_for_window(window)
                .is_some_and(|w| !w.tab_strip_active_visibility.is_empty()),
            "latest snapshot should persist even when the current frame snapshot is reset"
        );

        assert!(
            store
                .workspace_for_window(window, FrameId(2))
                .is_some_and(|w| w.tab_strip_active_visibility.is_empty()),
            "frame-scoped snapshot should be cleared by begin_frame when not recorded"
        );
    }
}
