use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use fret_core::{AppWindowId, FrameId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RunnerFrameDriveReason {
    EffectRedraw,
    EffectRequestAnimationFrame,
    AboutToWaitRaf,
    StreamingPendingRedrawAll,
    StreamingPendingRedrawWindow,
    SurfaceBootstrap,
    SurfaceRecoverLost,
    SurfaceRecoverOutdated,
    SurfaceRecoverTimeout,
    WebDiagKeepaliveRedraw,
}

impl RunnerFrameDriveReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EffectRedraw => "effect_redraw",
            Self::EffectRequestAnimationFrame => "effect_request_animation_frame",
            Self::AboutToWaitRaf => "about_to_wait_raf",
            Self::StreamingPendingRedrawAll => "streaming_pending_redraw_all",
            Self::StreamingPendingRedrawWindow => "streaming_pending_redraw_window",
            Self::SurfaceBootstrap => "surface_bootstrap",
            Self::SurfaceRecoverLost => "surface_recover_lost",
            Self::SurfaceRecoverOutdated => "surface_recover_outdated",
            Self::SurfaceRecoverTimeout => "surface_recover_timeout",
            Self::WebDiagKeepaliveRedraw => "web_diag_keepalive_redraw",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnerFrameDriveReasonCount {
    pub reason: RunnerFrameDriveReason,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunnerFrameDriveWindowSnapshot {
    pub total_event_count: u64,
    pub last_event_frame_id: u64,
    pub last_event_unix_ms: Option<u64>,
    pub reason_counts: Vec<RunnerFrameDriveReasonCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunnerFrameDriveAggregateSnapshot {
    pub window_count: u32,
    pub total_event_count: u64,
    pub max_event_count: u64,
    pub last_event_frame_id: u64,
    pub last_event_unix_ms: Option<u64>,
    pub reason_counts: Vec<RunnerFrameDriveReasonCount>,
}

#[derive(Debug, Default)]
struct RunnerFrameDriveWindowState {
    total_event_count: u64,
    last_event_frame_id: u64,
    last_event_unix_ms: Option<u64>,
    reason_counts: BTreeMap<RunnerFrameDriveReason, u64>,
}

impl RunnerFrameDriveWindowState {
    fn snapshot(&self) -> RunnerFrameDriveWindowSnapshot {
        RunnerFrameDriveWindowSnapshot {
            total_event_count: self.total_event_count,
            last_event_frame_id: self.last_event_frame_id,
            last_event_unix_ms: self.last_event_unix_ms,
            reason_counts: self
                .reason_counts
                .iter()
                .map(|(reason, count)| RunnerFrameDriveReasonCount {
                    reason: *reason,
                    count: *count,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Default)]
struct RunnerFrameDriveDiagnosticsState {
    windows: HashMap<AppWindowId, RunnerFrameDriveWindowState>,
}

#[derive(Debug, Clone, Default)]
pub struct RunnerFrameDriveDiagnosticsStore {
    inner: Arc<Mutex<RunnerFrameDriveDiagnosticsState>>,
}

impl RunnerFrameDriveDiagnosticsStore {
    pub fn window_snapshot(&self, window: AppWindowId) -> Option<RunnerFrameDriveWindowSnapshot> {
        self.inner
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .windows
            .get(&window)
            .map(RunnerFrameDriveWindowState::snapshot)
    }

    pub fn aggregate_snapshot(&self) -> RunnerFrameDriveAggregateSnapshot {
        let state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let mut reason_counts = BTreeMap::<RunnerFrameDriveReason, u64>::new();
        let mut snapshot = RunnerFrameDriveAggregateSnapshot {
            window_count: state.windows.len().min(u32::MAX as usize) as u32,
            ..RunnerFrameDriveAggregateSnapshot::default()
        };

        for window_snapshot in state.windows.values() {
            snapshot.total_event_count = snapshot
                .total_event_count
                .saturating_add(window_snapshot.total_event_count);
            snapshot.max_event_count = snapshot
                .max_event_count
                .max(window_snapshot.total_event_count);
            if window_snapshot.last_event_unix_ms.unwrap_or(0)
                >= snapshot.last_event_unix_ms.unwrap_or(0)
            {
                snapshot.last_event_unix_ms = window_snapshot.last_event_unix_ms;
                snapshot.last_event_frame_id = window_snapshot.last_event_frame_id;
            }
            for (reason, count) in &window_snapshot.reason_counts {
                reason_counts
                    .entry(*reason)
                    .and_modify(|value| *value = value.saturating_add(*count))
                    .or_insert(*count);
            }
        }

        snapshot.reason_counts = reason_counts
            .into_iter()
            .map(|(reason, count)| RunnerFrameDriveReasonCount { reason, count })
            .collect();
        snapshot
    }

    pub fn record(&self, window: AppWindowId, frame_id: FrameId, reason: RunnerFrameDriveReason) {
        let mut state = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        let entry = state.windows.entry(window).or_default();
        entry.total_event_count = entry.total_event_count.saturating_add(1);
        entry.last_event_frame_id = frame_id.0;
        entry.last_event_unix_ms = Some(unix_ms_now());
        entry
            .reason_counts
            .entry(reason)
            .and_modify(|count| *count = count.saturating_add(1))
            .or_insert(1);
    }

    pub fn clear_window(&self, window: AppWindowId) -> Option<RunnerFrameDriveWindowSnapshot> {
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
    fn record_updates_window_and_aggregate_reason_counts() {
        let store = RunnerFrameDriveDiagnosticsStore::default();
        let w1 = AppWindowId::from(KeyData::from_ffi(1));
        let w2 = AppWindowId::from(KeyData::from_ffi(2));

        store.record(w1, FrameId(3), RunnerFrameDriveReason::EffectRedraw);
        store.record(
            w1,
            FrameId(4),
            RunnerFrameDriveReason::EffectRequestAnimationFrame,
        );
        store.record(w2, FrameId(8), RunnerFrameDriveReason::AboutToWaitRaf);

        let w1_snapshot = store.window_snapshot(w1).expect("window snapshot");
        assert_eq!(w1_snapshot.total_event_count, 2);
        assert_eq!(w1_snapshot.last_event_frame_id, 4);
        assert!(w1_snapshot.last_event_unix_ms.is_some());
        assert_eq!(w1_snapshot.reason_counts.len(), 2);

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 2);
        assert_eq!(aggregate.total_event_count, 3);
        assert_eq!(aggregate.max_event_count, 2);
        assert!(matches!(aggregate.last_event_frame_id, 4 | 8));
        assert!(aggregate.last_event_unix_ms.is_some());
        assert_eq!(aggregate.reason_counts.len(), 3);
    }

    #[test]
    fn clear_window_removes_it_from_aggregate_snapshot() {
        let store = RunnerFrameDriveDiagnosticsStore::default();
        let window = AppWindowId::from(KeyData::from_ffi(7));
        store.record(window, FrameId(11), RunnerFrameDriveReason::EffectRedraw);
        assert!(store.window_snapshot(window).is_some());

        let removed = store.clear_window(window).expect("removed snapshot");
        assert_eq!(removed.total_event_count, 1);
        assert!(store.window_snapshot(window).is_none());

        let aggregate = store.aggregate_snapshot();
        assert_eq!(aggregate.window_count, 0);
        assert_eq!(aggregate.total_event_count, 0);
        assert_eq!(aggregate.max_event_count, 0);
        assert_eq!(aggregate.last_event_unix_ms, None);
        assert!(aggregate.reason_counts.is_empty());
    }
}
