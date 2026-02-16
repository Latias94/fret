use std::collections::HashMap;

use fret_core::{AppWindowId, ClipboardToken, FrameId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardReadDiagnostics {
    pub token: ClipboardToken,
    pub unavailable: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardWriteDiagnostics {
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
    last_write: Option<ClipboardWriteDiagnostics>,
}

impl WindowClipboardDiagnosticsStore {
    pub fn record_read_ok(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        token: ClipboardToken,
    ) {
        let entry = self.per_window.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
            entry.last_write = None;
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
        let entry = self.per_window.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
            entry.last_write = None;
        }
        entry.last_read = Some(ClipboardReadDiagnostics {
            token,
            unavailable: true,
            message,
        });
    }

    pub fn record_write_ok(&mut self, window: AppWindowId, frame_id: FrameId) {
        let entry = self.per_window.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
            entry.last_write = None;
        }
        entry.last_write = Some(ClipboardWriteDiagnostics {
            unavailable: false,
            message: None,
        });
    }

    pub fn record_write_unavailable(
        &mut self,
        window: AppWindowId,
        frame_id: FrameId,
        message: Option<String>,
    ) {
        let entry = self.per_window.entry(window).or_default();
        if entry.frame_id != frame_id {
            entry.frame_id = frame_id;
            entry.last_read = None;
            entry.last_write = None;
        }
        entry.last_write = Some(ClipboardWriteDiagnostics {
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

    pub fn last_write_for_window(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
    ) -> Option<&ClipboardWriteDiagnostics> {
        self.per_window
            .get(&window)
            .filter(|entry| entry.frame_id == frame_id)
            .and_then(|entry| entry.last_write.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_diagnostics_reset_per_frame_and_coexist_with_read() {
        let window = AppWindowId::default();
        let token = ClipboardToken::default();
        let frame_1 = FrameId(1);
        let frame_2 = FrameId(2);

        let mut store = WindowClipboardDiagnosticsStore::default();
        store.record_write_unavailable(window, frame_1, Some("nope".to_string()));

        let write_1 = store
            .last_write_for_window(window, frame_1)
            .expect("last_write frame 1");
        assert!(write_1.unavailable);
        assert_eq!(write_1.message.as_deref(), Some("nope"));

        assert!(store.last_write_for_window(window, frame_2).is_none());

        store.record_write_ok(window, frame_2);
        let write_2 = store
            .last_write_for_window(window, frame_2)
            .expect("last_write frame 2");
        assert!(!write_2.unavailable);
        assert!(write_2.message.is_none());

        // Recording read diagnostics in the same frame should not clear the write entry.
        store.record_read_ok(window, frame_2, token);
        assert!(store.last_read_for_window(window, frame_2).is_some());
        assert!(store.last_write_for_window(window, frame_2).is_some());
    }
}
