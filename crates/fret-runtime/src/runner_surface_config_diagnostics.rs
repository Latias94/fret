use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use fret_core::{
    time::{SystemTime, UNIX_EPOCH},
    AppWindowId, FrameId,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunnerSurfaceConfigWindowSnapshot {
    pub width_px: u32,
    pub height_px: u32,
    pub format: String,
    pub present_mode: String,
    pub desired_maximum_frame_latency: u32,
    pub alpha_mode: String,
    pub configure_count: u64,
    pub last_configure_frame_id: u64,
    pub last_configure_unix_ms: Option<u64>,
}

#[derive(Debug, Default)]
struct RunnerSurfaceConfigDiagnosticsState {
    windows: HashMap<AppWindowId, RunnerSurfaceConfigWindowSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct RunnerSurfaceConfigDiagnosticsStore {
    inner: Arc<Mutex<RunnerSurfaceConfigDiagnosticsState>>,
}

impl RunnerSurfaceConfigDiagnosticsStore {
    #[allow(clippy::too_many_arguments)]
    pub fn record_config(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
        width_px: u32,
        height_px: u32,
        format: impl Into<String>,
        present_mode: impl Into<String>,
        desired_maximum_frame_latency: u32,
        alpha_mode: impl Into<String>,
    ) {
        let mut state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let entry = state.windows.entry(window).or_default();
        entry.width_px = width_px;
        entry.height_px = height_px;
        entry.format = format.into();
        entry.present_mode = present_mode.into();
        entry.desired_maximum_frame_latency = desired_maximum_frame_latency;
        entry.alpha_mode = alpha_mode.into();
        entry.configure_count = entry.configure_count.saturating_add(1);
        entry.last_configure_frame_id = frame_id.0;
        entry.last_configure_unix_ms = Some(unix_ms_now());
    }

    pub fn window_snapshot(
        &self,
        window: AppWindowId,
    ) -> Option<RunnerSurfaceConfigWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .get(&window)
            .cloned()
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<RunnerSurfaceConfigWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .remove(&window)
    }
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slotmap::KeyData;

    #[test]
    fn record_config_updates_window_snapshot() {
        let store = RunnerSurfaceConfigDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(7));

        store.record_config(
            window,
            FrameId(11),
            1000,
            800,
            "Bgra8UnormSrgb",
            "Fifo",
            2,
            "Opaque",
        );
        store.record_config(
            window,
            FrameId(12),
            1200,
            900,
            "Bgra8UnormSrgb",
            "Mailbox",
            3,
            "PreMultiplied",
        );

        let snapshot = store.window_snapshot(window).expect("surface snapshot");
        assert_eq!(snapshot.width_px, 1200);
        assert_eq!(snapshot.height_px, 900);
        assert_eq!(snapshot.format, "Bgra8UnormSrgb");
        assert_eq!(snapshot.present_mode, "Mailbox");
        assert_eq!(snapshot.desired_maximum_frame_latency, 3);
        assert_eq!(snapshot.alpha_mode, "PreMultiplied");
        assert_eq!(snapshot.configure_count, 2);
        assert_eq!(snapshot.last_configure_frame_id, 12);
        assert!(snapshot.last_configure_unix_ms.is_some());
    }

    #[test]
    fn clear_window_removes_snapshot() {
        let store = RunnerSurfaceConfigDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(9));
        store.record_config(window, FrameId(1), 1, 1, "fmt", "mode", 2, "alpha");
        assert!(store.window_snapshot(window).is_some());
        let removed = store.clear_window(window).expect("removed snapshot");
        assert_eq!(removed.configure_count, 1);
        assert!(store.window_snapshot(window).is_none());
    }
}
