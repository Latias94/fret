use std::collections::{BTreeMap, HashMap};
use std::panic::Location;
use std::sync::{Arc, Mutex};

use fret_core::{
    AppWindowId, FrameId,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RedrawRequestCallsiteKey {
    file: &'static str,
    line: u32,
    column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedrawRequestCallsiteCount {
    pub file: &'static str,
    pub line: u32,
    pub column: u32,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowRedrawRequestWindowSnapshot {
    pub total_request_count: u64,
    pub last_request_frame_id: u64,
    pub last_request_unix_ms: Option<u64>,
    pub callsites: Vec<RedrawRequestCallsiteCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowRedrawRequestAggregateSnapshot {
    pub window_count: u32,
    pub total_request_count: u64,
    pub max_request_count: u64,
    pub last_request_frame_id: u64,
    pub last_request_unix_ms: Option<u64>,
    pub callsites: Vec<RedrawRequestCallsiteCount>,
}

#[derive(Debug, Default)]
struct WindowRedrawRequestWindowState {
    total_request_count: u64,
    last_request_frame_id: u64,
    last_request_unix_ms: Option<u64>,
    callsites: BTreeMap<RedrawRequestCallsiteKey, u64>,
}

impl WindowRedrawRequestWindowState {
    fn snapshot(&self) -> WindowRedrawRequestWindowSnapshot {
        let mut callsites: Vec<_> = self
            .callsites
            .iter()
            .map(|(key, count)| RedrawRequestCallsiteCount {
                file: key.file,
                line: key.line,
                column: key.column,
                count: *count,
            })
            .collect();
        callsites.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| a.file.cmp(b.file))
                .then_with(|| a.line.cmp(&b.line))
                .then_with(|| a.column.cmp(&b.column))
        });
        WindowRedrawRequestWindowSnapshot {
            total_request_count: self.total_request_count,
            last_request_frame_id: self.last_request_frame_id,
            last_request_unix_ms: self.last_request_unix_ms,
            callsites,
        }
    }
}

#[derive(Debug, Default)]
struct WindowRedrawRequestDiagnosticsState {
    windows: HashMap<AppWindowId, WindowRedrawRequestWindowState>,
}

#[derive(Debug, Clone, Default)]
pub struct WindowRedrawRequestDiagnosticsStore {
    inner: Arc<Mutex<WindowRedrawRequestDiagnosticsState>>,
}

impl WindowRedrawRequestDiagnosticsStore {
    pub fn window_snapshot(
        &self,
        window: AppWindowId,
    ) -> Option<WindowRedrawRequestWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .get(&window)
            .map(WindowRedrawRequestWindowState::snapshot)
    }

    pub fn aggregate_snapshot(&self) -> WindowRedrawRequestAggregateSnapshot {
        let state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let mut callsites = BTreeMap::<RedrawRequestCallsiteKey, u64>::new();
        let mut snapshot = WindowRedrawRequestAggregateSnapshot {
            window_count: state.windows.len().min(u32::MAX as usize) as u32,
            ..WindowRedrawRequestAggregateSnapshot::default()
        };

        for window_snapshot in state.windows.values() {
            snapshot.total_request_count = snapshot
                .total_request_count
                .saturating_add(window_snapshot.total_request_count);
            snapshot.max_request_count = snapshot
                .max_request_count
                .max(window_snapshot.total_request_count);
            let next_unix_ms = window_snapshot.last_request_unix_ms.unwrap_or(0);
            let current_unix_ms = snapshot.last_request_unix_ms.unwrap_or(0);
            if next_unix_ms > current_unix_ms
                || (next_unix_ms == current_unix_ms
                    && window_snapshot.last_request_frame_id >= snapshot.last_request_frame_id)
            {
                snapshot.last_request_unix_ms = window_snapshot.last_request_unix_ms;
                snapshot.last_request_frame_id = window_snapshot.last_request_frame_id;
            }
            for (key, count) in &window_snapshot.callsites {
                callsites
                    .entry(*key)
                    .and_modify(|value| *value = value.saturating_add(*count))
                    .or_insert(*count);
            }
        }

        let mut aggregated_callsites: Vec<_> = callsites
            .into_iter()
            .map(|(key, count)| RedrawRequestCallsiteCount {
                file: key.file,
                line: key.line,
                column: key.column,
                count,
            })
            .collect();
        aggregated_callsites.sort_by(|a, b| {
            b.count
                .cmp(&a.count)
                .then_with(|| a.file.cmp(b.file))
                .then_with(|| a.line.cmp(&b.line))
                .then_with(|| a.column.cmp(&b.column))
        });
        snapshot.callsites = aggregated_callsites;
        snapshot
    }

    pub fn record(
        &self,
        window: AppWindowId,
        frame_id: FrameId,
        location: &'static Location<'static>,
    ) {
        let mut state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let entry = state.windows.entry(window).or_default();
        entry.total_request_count = entry.total_request_count.saturating_add(1);
        entry.last_request_frame_id = frame_id.0;
        entry.last_request_unix_ms = Some(unix_ms_now());
        let key = RedrawRequestCallsiteKey {
            file: location.file(),
            line: location.line(),
            column: location.column(),
        };
        entry
            .callsites
            .entry(key)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<WindowRedrawRequestWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .remove(&window)
            .map(|state| state.snapshot())
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

    fn record_here(
        store: &WindowRedrawRequestDiagnosticsStore,
        window: AppWindowId,
        frame_id: FrameId,
    ) {
        store.record(window, frame_id, Location::caller());
    }

    fn record_other(
        store: &WindowRedrawRequestDiagnosticsStore,
        window: AppWindowId,
        frame_id: FrameId,
    ) {
        store.record(window, frame_id, Location::caller());
    }

    #[test]
    fn record_aggregates_callsites() {
        let store = WindowRedrawRequestDiagnosticsStore::default();
        let w1 = AppWindowId::from(KeyData::from_ffi(1));
        let w2 = AppWindowId::from(KeyData::from_ffi(2));

        record_here(&store, w1, FrameId(3));
        record_here(&store, w1, FrameId(4));
        record_other(&store, w2, FrameId(8));

        let w1_snapshot = store.window_snapshot(w1).expect("window snapshot");
        assert_eq!(w1_snapshot.total_request_count, 2);
        assert_eq!(w1_snapshot.last_request_frame_id, 4);
        assert!(w1_snapshot.last_request_unix_ms.is_some());
        assert_eq!(w1_snapshot.callsites.len(), 1);
        assert_eq!(w1_snapshot.callsites[0].count, 2);

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 2);
        assert_eq!(aggregate.total_request_count, 3);
        assert_eq!(aggregate.max_request_count, 2);
        assert_eq!(aggregate.last_request_frame_id, 8);
        assert!(aggregate.last_request_unix_ms.is_some());
        assert_eq!(aggregate.callsites.len(), 2);
        assert_eq!(aggregate.callsites[0].count, 2);
    }

    #[test]
    fn clear_window_removes_callsites() {
        let store = WindowRedrawRequestDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(7));
        record_here(&store, window, FrameId(11));

        let removed = store.clear_window(window).expect("removed snapshot");
        assert_eq!(removed.total_request_count, 1);
        assert_eq!(removed.callsites.len(), 1);
        assert!(store.window_snapshot(window).is_none());

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 0);
        assert_eq!(aggregate.total_request_count, 0);
        assert_eq!(aggregate.max_request_count, 0);
        assert_eq!(aggregate.last_request_unix_ms, None);
        assert!(aggregate.callsites.is_empty());
    }
}
