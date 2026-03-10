use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use fret_core::{
    AppWindowId, FrameId,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunnerPresentWindowSnapshot {
    pub present_count: u64,
    pub last_present_frame_id: u64,
    pub last_present_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunnerPresentAggregateSnapshot {
    pub window_count: u32,
    pub total_present_count: u64,
    pub max_present_count: u64,
    pub last_present_frame_id: u64,
    pub last_present_unix_ms: Option<u64>,
}

#[derive(Debug, Default)]
struct RunnerPresentDiagnosticsState {
    windows: HashMap<AppWindowId, RunnerPresentWindowSnapshot>,
}

#[derive(Debug, Clone, Default)]
pub struct RunnerPresentDiagnosticsStore {
    inner: Arc<Mutex<RunnerPresentDiagnosticsState>>,
}

impl RunnerPresentDiagnosticsStore {
    pub fn window_snapshot(&self, window: AppWindowId) -> Option<RunnerPresentWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .get(&window)
            .copied()
    }

    pub fn aggregate_snapshot(&self) -> RunnerPresentAggregateSnapshot {
        let state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let mut snapshot = RunnerPresentAggregateSnapshot {
            window_count: state.windows.len().min(u32::MAX as usize) as u32,
            ..RunnerPresentAggregateSnapshot::default()
        };

        for window_snapshot in state.windows.values() {
            snapshot.total_present_count = snapshot
                .total_present_count
                .saturating_add(window_snapshot.present_count);
            snapshot.max_present_count = snapshot
                .max_present_count
                .max(window_snapshot.present_count);
            if window_snapshot.last_present_unix_ms.unwrap_or(0)
                >= snapshot.last_present_unix_ms.unwrap_or(0)
            {
                snapshot.last_present_unix_ms = window_snapshot.last_present_unix_ms;
                snapshot.last_present_frame_id = window_snapshot.last_present_frame_id;
            }
        }

        snapshot
    }

    pub fn record_present(&self, window: AppWindowId, frame_id: FrameId) {
        let mut state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let entry = state.windows.entry(window).or_default();
        entry.present_count = entry.present_count.saturating_add(1);
        entry.last_present_frame_id = frame_id.0;
        entry.last_present_unix_ms = Some(unix_ms_now());
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<RunnerPresentWindowSnapshot> {
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
    fn record_present_updates_window_and_aggregate_snapshots() {
        let store = RunnerPresentDiagnosticsStore::default();
        let w1 = AppWindowId::from(KeyData::from_ffi(1));
        let w2 = AppWindowId::from(KeyData::from_ffi(2));

        store.record_present(w1, FrameId(3));
        store.record_present(w1, FrameId(4));
        store.record_present(w2, FrameId(8));

        let w1_snapshot = store.window_snapshot(w1).expect("window snapshot");
        assert_eq!(w1_snapshot.present_count, 2);
        assert_eq!(w1_snapshot.last_present_frame_id, 4);
        assert!(w1_snapshot.last_present_unix_ms.is_some());

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 2);
        assert_eq!(aggregate.total_present_count, 3);
        assert_eq!(aggregate.max_present_count, 2);
        assert_eq!(aggregate.last_present_frame_id, 8);
        assert!(aggregate.last_present_unix_ms.is_some());
    }

    #[test]
    fn clear_window_removes_it_from_aggregate_snapshot() {
        let store = RunnerPresentDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(7));
        store.record_present(window, FrameId(11));
        assert!(store.window_snapshot(window).is_some());

        let removed = store.clear_window(window).expect("removed snapshot");
        assert_eq!(removed.present_count, 1);
        assert!(store.window_snapshot(window).is_none());

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 0);
        assert_eq!(aggregate.total_present_count, 0);
        assert_eq!(aggregate.max_present_count, 0);
        assert_eq!(aggregate.last_present_unix_ms, None);
    }
}
