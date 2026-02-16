use std::collections::HashMap;

use fret_core::{AppWindowId, ClipboardToken, FrameId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardReadDiagnostics {
    pub token: ClipboardToken,
    pub unavailable: bool,
    pub message: Option<String>,
}

#[derive(Debug, Default)]
pub struct WindowClipboardDiagnosticsStore {
    per_window: HashMap<AppWindowId, WindowClipboardDiagnosticsFrame>,
}

#[derive(Debug, Default)]
struct WindowClipboardDiagnosticsFrame {
    frame_id: FrameId,
    last_read: Option<ClipboardReadDiagnostics>,
}

impl WindowClipboardDiagnosticsStore {
    pub fn record_read_ok(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        token: ClipboardToken,
    ) {
        let entry = self
            .per_window
            .entry(window)
            .or_insert_with(WindowClipboardDiagnosticsFrame::default);
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
        }
        entry.last_read = Some(ClipboardReadDiagnostics {
            token,
            unavailable: false,
            message: None,
        });
    }

    pub fn record_read_unavailable(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        token: ClipboardToken,
        message: Option<String>,
    ) {
        let entry = self
            .per_window
            .entry(window)
            .or_insert_with(WindowClipboardDiagnosticsFrame::default);
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
        }
        entry.last_read = Some(ClipboardReadDiagnostics {
            token,
            unavailable: true,
            message,
        });
    }

    pub fn last_read_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&ClipboardReadDiagnostics> {
        self.per_window
            .get(&window)
            .filter(|entry| entry.frame_id == frame_id)
            .and_then(|entry| entry.last_read.as_ref())
    }
}
