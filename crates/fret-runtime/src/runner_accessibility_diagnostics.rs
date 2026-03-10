use std::collections::HashMap;

use fret_core::{
    AppWindowId, FrameId,
    time::{SystemTime, UNIX_EPOCH},
};

/// Diagnostics store for runner accessibility activation events.
///
/// This is intended as a lightweight evidence anchor that the AccessKit platform adapter has been
/// activated by the OS accessibility stack (e.g. macOS AX / VoiceOver), rather than only producing
/// an internal semantics tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunnerAccessibilitySnapshot {
    pub activation_requests: u64,
    pub last_activation_unix_ms: Option<u64>,
    pub last_activation_frame_id: Option<FrameId>,
}

#[derive(Debug, Default)]
pub struct RunnerAccessibilityDiagnosticsStore {
    per_window: HashMap<AppWindowId, RunnerAccessibilitySnapshot>,
}

impl RunnerAccessibilityDiagnosticsStore {
    pub fn snapshot(&self, window: AppWindowId) -> Option<RunnerAccessibilitySnapshot> {
        self.per_window.get(&window).copied()
    }

    pub fn record_activation_request(&mut self, window: AppWindowId, frame_id: FrameId) {
        let entry = self.per_window.entry(window).or_default();
        entry.activation_requests = entry.activation_requests.saturating_add(1);
        entry.last_activation_unix_ms = Some(unix_ms_now());
        entry.last_activation_frame_id = Some(frame_id);
    }
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
