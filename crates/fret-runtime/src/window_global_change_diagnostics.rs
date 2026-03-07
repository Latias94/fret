use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use fret_core::{AppWindowId, FrameId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowGlobalChangeNameCount {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowGlobalChangeWindowSnapshot {
    pub batch_count: u64,
    pub total_global_count: u64,
    pub last_frame_id: u64,
    pub last_unix_ms: Option<u64>,
    pub globals: Vec<WindowGlobalChangeNameCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowGlobalChangeAggregateSnapshot {
    pub window_count: u32,
    pub batch_count: u64,
    pub total_global_count: u64,
    pub max_batch_count: u64,
    pub last_frame_id: u64,
    pub last_unix_ms: Option<u64>,
    pub globals: Vec<WindowGlobalChangeNameCount>,
}

#[derive(Debug, Default)]
struct WindowGlobalChangeWindowState {
    batch_count: u64,
    total_global_count: u64,
    last_frame_id: u64,
    last_unix_ms: Option<u64>,
    globals: HashMap<String, u64>,
}

impl WindowGlobalChangeWindowState {
    fn snapshot(&self) -> WindowGlobalChangeWindowSnapshot {
        let mut globals: Vec<_> = self
            .globals
            .iter()
            .map(|(name, count)| WindowGlobalChangeNameCount {
                name: name.clone(),
                count: *count,
            })
            .collect();
        globals.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        WindowGlobalChangeWindowSnapshot {
            batch_count: self.batch_count,
            total_global_count: self.total_global_count,
            last_frame_id: self.last_frame_id,
            last_unix_ms: self.last_unix_ms,
            globals,
        }
    }
}

#[derive(Debug, Default)]
struct WindowGlobalChangeDiagnosticsState {
    windows: HashMap<AppWindowId, WindowGlobalChangeWindowState>,
}

#[derive(Debug, Clone, Default)]
pub struct WindowGlobalChangeDiagnosticsStore {
    inner: Arc<Mutex<WindowGlobalChangeDiagnosticsState>>,
}

impl WindowGlobalChangeDiagnosticsStore {
    pub fn window_snapshot(&self, window: AppWindowId) -> Option<WindowGlobalChangeWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .get(&window)
            .map(WindowGlobalChangeWindowState::snapshot)
    }

    pub fn aggregate_snapshot(&self) -> WindowGlobalChangeAggregateSnapshot {
        let state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let mut globals = HashMap::<String, u64>::new();
        let mut snapshot = WindowGlobalChangeAggregateSnapshot {
            window_count: state.windows.len().min(u32::MAX as usize) as u32,
            ..WindowGlobalChangeAggregateSnapshot::default()
        };

        for window_snapshot in state.windows.values() {
            snapshot.batch_count = snapshot
                .batch_count
                .saturating_add(window_snapshot.batch_count);
            snapshot.total_global_count = snapshot
                .total_global_count
                .saturating_add(window_snapshot.total_global_count);
            snapshot.max_batch_count = snapshot.max_batch_count.max(window_snapshot.batch_count);
            if window_snapshot.last_unix_ms.unwrap_or(0) >= snapshot.last_unix_ms.unwrap_or(0) {
                snapshot.last_unix_ms = window_snapshot.last_unix_ms;
                snapshot.last_frame_id = window_snapshot.last_frame_id;
            }
            for (name, count) in &window_snapshot.globals {
                globals
                    .entry(name.clone())
                    .and_modify(|value| *value = value.saturating_add(*count))
                    .or_insert(*count);
            }
        }

        let mut aggregate_globals: Vec<_> = globals
            .into_iter()
            .map(|(name, count)| WindowGlobalChangeNameCount { name, count })
            .collect();
        aggregate_globals.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
        snapshot.globals = aggregate_globals;
        snapshot
    }

    pub fn record_batch<I>(&self, window: AppWindowId, frame_id: FrameId, names: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let entry = state.windows.entry(window).or_default();
        entry.batch_count = entry.batch_count.saturating_add(1);
        entry.last_frame_id = frame_id.0;
        entry.last_unix_ms = Some(unix_ms_now());
        for name in names {
            entry.total_global_count = entry.total_global_count.saturating_add(1);
            let name = name.as_ref();
            entry
                .globals
                .entry(name.to_string())
                .and_modify(|count| *count = count.saturating_add(1))
                .or_insert(1);
        }
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<WindowGlobalChangeWindowSnapshot> {
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

    #[test]
    fn record_batch_aggregates_global_names() {
        let store = WindowGlobalChangeDiagnosticsStore::default();
        let w1 = AppWindowId::from(KeyData::from_ffi(1));
        let w2 = AppWindowId::from(KeyData::from_ffi(2));

        store.record_batch(w1, FrameId(3), ["A", "B"]);
        store.record_batch(w1, FrameId(4), ["A"]);
        store.record_batch(w2, FrameId(8), ["C"]);

        let w1_snapshot = store.window_snapshot(w1).expect("window snapshot");
        assert_eq!(w1_snapshot.batch_count, 2);
        assert_eq!(w1_snapshot.total_global_count, 3);
        assert_eq!(w1_snapshot.globals[0].name, "A");
        assert_eq!(w1_snapshot.globals[0].count, 2);

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 2);
        assert_eq!(aggregate.batch_count, 3);
        assert_eq!(aggregate.total_global_count, 4);
        assert_eq!(aggregate.max_batch_count, 2);
        assert!(matches!(aggregate.last_frame_id, 4 | 8));
        assert_eq!(aggregate.globals[0].name, "A");
        assert_eq!(aggregate.globals[0].count, 2);
    }

    #[test]
    fn clear_window_removes_global_counts() {
        let store = WindowGlobalChangeDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(7));
        store.record_batch(window, FrameId(11), ["A"]);

        let removed = store.clear_window(window).expect("removed snapshot");
        assert_eq!(removed.batch_count, 1);
        assert_eq!(removed.total_global_count, 1);
        assert!(store.window_snapshot(window).is_none());

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 0);
        assert_eq!(aggregate.batch_count, 0);
        assert_eq!(aggregate.total_global_count, 0);
        assert_eq!(aggregate.max_batch_count, 0);
        assert_eq!(aggregate.last_unix_ms, None);
        assert!(aggregate.globals.is_empty());
    }
}
