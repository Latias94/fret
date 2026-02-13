use std::time::{SystemTime, UNIX_EPOCH};

/// Diagnostics store for runner surface lifecycle transitions.
///
/// This is intended to help mobile bring-up verify that winit lifecycle hooks are firing and that
/// surfaces are dropped/recreated as expected on background/foreground transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunnerSurfaceLifecycleSnapshot {
    pub can_create_surfaces_calls: u64,
    pub destroy_surfaces_calls: u64,
    pub last_can_create_surfaces_unix_ms: Option<u64>,
    pub last_destroy_surfaces_unix_ms: Option<u64>,
    pub surfaces_available: bool,
}

#[derive(Debug, Default)]
pub struct RunnerSurfaceLifecycleDiagnosticsStore {
    snapshot: RunnerSurfaceLifecycleSnapshot,
}

impl RunnerSurfaceLifecycleDiagnosticsStore {
    pub fn snapshot(&self) -> RunnerSurfaceLifecycleSnapshot {
        self.snapshot
    }

    pub fn record_can_create_surfaces(&mut self) {
        self.snapshot.can_create_surfaces_calls =
            self.snapshot.can_create_surfaces_calls.saturating_add(1);
        self.snapshot.last_can_create_surfaces_unix_ms = Some(unix_ms_now());
        self.snapshot.surfaces_available = true;
    }

    pub fn record_destroy_surfaces(&mut self) {
        self.snapshot.destroy_surfaces_calls =
            self.snapshot.destroy_surfaces_calls.saturating_add(1);
        self.snapshot.last_destroy_surfaces_unix_ms = Some(unix_ms_now());
        self.snapshot.surfaces_available = false;
    }
}

fn unix_ms_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
