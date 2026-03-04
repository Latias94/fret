use fret_core::AppWindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerPlatformWindowReceiverAtCursorSourceV1 {
    Unsupported,
    Win32WindowFromPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerPlatformWindowReceiverAtCursorSnapshotV1 {
    pub receiver_window: Option<AppWindowId>,
    pub source: RunnerPlatformWindowReceiverAtCursorSourceV1,
}

#[derive(Debug, Default)]
pub struct RunnerPlatformWindowReceiverDiagnosticsStore {
    latest_at_cursor: Option<RunnerPlatformWindowReceiverAtCursorSnapshotV1>,
}

impl RunnerPlatformWindowReceiverDiagnosticsStore {
    pub fn latest_at_cursor(&self) -> Option<RunnerPlatformWindowReceiverAtCursorSnapshotV1> {
        self.latest_at_cursor
    }

    pub fn set_latest_at_cursor(
        &mut self,
        snapshot: RunnerPlatformWindowReceiverAtCursorSnapshotV1,
    ) {
        self.latest_at_cursor = Some(snapshot);
    }
}
