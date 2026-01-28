use std::collections::HashMap;

use fret_core::{AppWindowId, PointerId, RenderTargetId};

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DockingInteractionDiagnostics {
    pub dock_drag: Option<DockDragDiagnostics>,
    pub viewport_capture: Option<ViewportCaptureDiagnostics>,
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
