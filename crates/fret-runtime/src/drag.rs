use std::any::Any;

use fret_core::{AppWindowId, Point, PointerId};

/// Best-effort diagnostics hint: which mechanism was used to select the hovered window during a
/// cross-window drag session.
///
/// This is primarily intended for multi-window docking diagnostics ("hovered window under cursor"
/// selection under overlap), so bundles can answer whether the runner used an OS-backed path or a
/// heuristic fallback.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowUnderCursorSource {
    /// No source information is available (or the runner has not attempted selection yet).
    #[default]
    Unknown,
    /// OS-backed Win32 window-under-cursor selection (z-order traversal).
    PlatformWin32,
    /// OS-backed macOS window-under-cursor selection.
    PlatformMacos,
    /// Stable latch (reuse the previously hovered window while the cursor remains inside it).
    Latched,
    /// Runner-maintained z-order list / rect scan (best-effort heuristic).
    HeuristicZOrder,
    /// Full window-rect scan (best-effort heuristic).
    HeuristicRects,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragKindId(pub u64);

pub const DRAG_KIND_DOCK_PANEL: DragKindId = DragKindId(1);
pub const DRAG_KIND_DOCK_TABS: DragKindId = DragKindId(2);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DragSessionId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragPhase {
    Starting,
    Dragging,
    Dropped,
    Canceled,
}

#[derive(Debug)]
pub struct DragSession {
    pub session_id: DragSessionId,
    pub pointer_id: PointerId,
    pub source_window: AppWindowId,
    pub current_window: AppWindowId,
    pub cross_window_hover: bool,
    pub kind: DragKindId,
    pub start_position: Point,
    pub position: Point,
    /// Cursor grab offset in window-local logical coordinates.
    ///
    /// Runners may use this to keep an OS window under the cursor during docking interactions
    /// (ImGui-style multi-viewport behavior), without needing to downcast the typed payload.
    pub cursor_grab_offset: Option<Point>,
    /// If set, requests the runner to treat this drag as a "move the OS window" interaction for
    /// the given window id, while still allowing cross-window docking hover/drop routing.
    pub follow_window: Option<AppWindowId>,
    /// Best-effort diagnostics hint: OS window currently being moved by the runner for this drag
    /// session (ImGui-style "follow window" multi-viewport behavior).
    ///
    /// This is intentionally diagnostics-only: it does not request follow behavior; it records
    /// what the runner is currently doing.
    pub moving_window: Option<AppWindowId>,
    /// Best-effort diagnostics hint: when [`Self::moving_window`] is set, the window considered
    /// "under" the moving window at the current cursor position.
    ///
    /// This exists to support ImGui-style terminology where `HoveredWindow` and
    /// `HoveredWindowUnderMovingWindow` can differ during a viewport drag. Fret currently keeps
    /// `current_window` as the runner-selected hover/drop target; this field makes it possible to
    /// gate and evolve "peek-behind" behavior without reinterpreting `current_window`.
    pub window_under_moving_window: Option<AppWindowId>,
    /// Best-effort diagnostics hint: which mechanism was used to select
    /// [`Self::window_under_moving_window`].
    pub window_under_moving_window_source: WindowUnderCursorSource,
    /// Best-effort diagnostics hint: true when the runner has applied an ImGui-style "transparent
    /// payload" treatment to the moving dock window (e.g. reduced opacity and/or click-through
    /// hit-test passthrough while following the cursor).
    pub transparent_payload_applied: bool,
    /// Best-effort diagnostics hint: true when the runner successfully applied click-through
    /// hit-test passthrough to the moving dock window while transparent payload is enabled.
    ///
    /// This is a result signal (applied by the OS/window backend), not a request. When false,
    /// the runner either did not attempt passthrough or the platform/backend rejected it.
    pub transparent_payload_hit_test_passthrough_applied: bool,
    /// Best-effort diagnostics hint: which mechanism was used to select the hovered window during
    /// cross-window drag routing (OS-backed vs heuristic).
    pub window_under_cursor_source: WindowUnderCursorSource,
    pub dragging: bool,
    pub phase: DragPhase,
    payload: Box<dyn Any>,
}

impl DragSession {
    pub fn new<T: Any>(
        session_id: DragSessionId,
        pointer_id: PointerId,
        source_window: AppWindowId,
        kind: DragKindId,
        start_position: Point,
        payload: T,
    ) -> Self {
        Self {
            session_id,
            pointer_id,
            source_window,
            current_window: source_window,
            cross_window_hover: false,
            kind,
            start_position,
            position: start_position,
            cursor_grab_offset: None,
            follow_window: None,
            moving_window: None,
            window_under_moving_window: None,
            window_under_moving_window_source: WindowUnderCursorSource::Unknown,
            transparent_payload_applied: false,
            transparent_payload_hit_test_passthrough_applied: false,
            window_under_cursor_source: WindowUnderCursorSource::Unknown,
            dragging: false,
            phase: DragPhase::Starting,
            payload: Box::new(payload),
        }
    }

    pub fn new_cross_window<T: Any>(
        session_id: DragSessionId,
        pointer_id: PointerId,
        source_window: AppWindowId,
        kind: DragKindId,
        start_position: Point,
        payload: T,
    ) -> Self {
        Self {
            session_id,
            pointer_id,
            source_window,
            current_window: source_window,
            cross_window_hover: true,
            kind,
            start_position,
            position: start_position,
            cursor_grab_offset: None,
            follow_window: None,
            moving_window: None,
            window_under_moving_window: None,
            window_under_moving_window_source: WindowUnderCursorSource::Unknown,
            transparent_payload_applied: false,
            transparent_payload_hit_test_passthrough_applied: false,
            window_under_cursor_source: WindowUnderCursorSource::Unknown,
            dragging: false,
            phase: DragPhase::Starting,
            payload: Box::new(payload),
        }
    }

    pub fn payload<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }

    pub fn payload_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.payload.downcast_mut::<T>()
    }

    pub fn into_payload(self) -> Box<dyn Any> {
        self.payload
    }
}
