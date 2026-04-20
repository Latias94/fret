/// Runner-owned host monitor topology snapshot used by diagnostics and future environment-aware
/// tooling.
///
/// This intentionally stays data-only and runner-facing: the desktop runner publishes the latest
/// host monitor inventory here, while diagnostics/tooling may consume it later without depending on
/// `winit` monitor handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunnerMonitorRectPhysicalV1 {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunnerMonitorInfoV1 {
    pub bounds_physical: RunnerMonitorRectPhysicalV1,
    pub scale_factor: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RunnerMonitorTopologySnapshotV1 {
    pub virtual_desktop_bounds_physical: Option<RunnerMonitorRectPhysicalV1>,
    pub monitors: Vec<RunnerMonitorInfoV1>,
}

#[derive(Debug, Default)]
pub struct RunnerMonitorTopologyDiagnosticsStore {
    snapshot: Option<RunnerMonitorTopologySnapshotV1>,
}

impl RunnerMonitorTopologyDiagnosticsStore {
    pub fn snapshot(&self) -> Option<RunnerMonitorTopologySnapshotV1> {
        self.snapshot.clone()
    }

    pub fn update_snapshot(&mut self, snapshot: RunnerMonitorTopologySnapshotV1) -> bool {
        if self.snapshot.as_ref() == Some(&snapshot) {
            return false;
        }
        self.snapshot = Some(snapshot);
        true
    }

    pub fn clear_snapshot(&mut self) {
        self.snapshot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> RunnerMonitorRectPhysicalV1 {
        RunnerMonitorRectPhysicalV1 {
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn update_snapshot_detects_real_changes() {
        let mut store = RunnerMonitorTopologyDiagnosticsStore::default();
        let snapshot = RunnerMonitorTopologySnapshotV1 {
            virtual_desktop_bounds_physical: Some(rect(0, 0, 3200, 1080)),
            monitors: vec![
                RunnerMonitorInfoV1 {
                    bounds_physical: rect(0, 0, 1920, 1080),
                    scale_factor: 1.0,
                },
                RunnerMonitorInfoV1 {
                    bounds_physical: rect(1920, 0, 1280, 1024),
                    scale_factor: 1.25,
                },
            ],
        };

        assert!(store.update_snapshot(snapshot.clone()));
        assert_eq!(store.snapshot(), Some(snapshot.clone()));
        assert!(!store.update_snapshot(snapshot));
    }

    #[test]
    fn clear_snapshot_removes_last_topology() {
        let mut store = RunnerMonitorTopologyDiagnosticsStore::default();
        assert!(store.update_snapshot(RunnerMonitorTopologySnapshotV1 {
            virtual_desktop_bounds_physical: Some(rect(0, 0, 1920, 1080)),
            monitors: vec![RunnerMonitorInfoV1 {
                bounds_physical: rect(0, 0, 1920, 1080),
                scale_factor: 1.0,
            }],
        }));

        store.clear_snapshot();
        assert_eq!(store.snapshot(), None);
    }
}
