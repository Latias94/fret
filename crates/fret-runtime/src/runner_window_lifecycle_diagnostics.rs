use std::collections::HashSet;

use fret_core::{
    AppWindowId,
    time::{SystemTime, UNIX_EPOCH},
};

/// Diagnostics store for runner window lifecycle events (open/close).
///
/// Diagnostics scripts use window-count predicates (e.g. `known_window_count_is`) as a stable
/// signal that OS windows have been created/closed. Counting windows via input-context snapshots
/// is opportunistic (it depends on input dispatch), so the runner maintains a source-of-truth set
/// of open windows for deterministic scripted runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunnerWindowLifecycleSnapshot {
    pub open_window_count: u32,
    pub window_open_events: u64,
    pub window_close_events: u64,
    pub last_window_open_unix_ms: Option<u64>,
    pub last_window_close_unix_ms: Option<u64>,
}

#[derive(Debug, Default)]
pub struct RunnerWindowLifecycleDiagnosticsStore {
    open_windows: HashSet<AppWindowId>,
    snapshot: RunnerWindowLifecycleSnapshot,
}

impl RunnerWindowLifecycleDiagnosticsStore {
    pub fn snapshot(&self) -> RunnerWindowLifecycleSnapshot {
        self.snapshot
    }

    pub fn record_window_open(&mut self, window: AppWindowId) {
        let inserted = self.open_windows.insert(window);
        if inserted {
            self.snapshot.window_open_events = self.snapshot.window_open_events.saturating_add(1);
            self.snapshot.last_window_open_unix_ms = Some(unix_ms_now());
            self.snapshot.open_window_count = self.open_windows.len().min(u32::MAX as usize) as u32;
        }
    }

    pub fn record_window_close(&mut self, window: AppWindowId) {
        let removed = self.open_windows.remove(&window);
        if removed {
            self.snapshot.window_close_events = self.snapshot.window_close_events.saturating_add(1);
            self.snapshot.last_window_close_unix_ms = Some(unix_ms_now());
            self.snapshot.open_window_count = self.open_windows.len().min(u32::MAX as usize) as u32;
        }
    }
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
