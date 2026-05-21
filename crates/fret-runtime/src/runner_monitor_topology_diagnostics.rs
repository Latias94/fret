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

    pub fn snapshot_matches(&self, snapshot: &RunnerMonitorTopologySnapshotV1) -> bool {
        self.snapshot.as_ref() == Some(snapshot)
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

pub fn update_runner_monitor_topology_diagnostics(
    host: &mut impl crate::ui_host::GlobalsHost,
    snapshot: RunnerMonitorTopologySnapshotV1,
) -> bool {
    if host
        .global::<RunnerMonitorTopologyDiagnosticsStore>()
        .is_some_and(|store| store.snapshot_matches(&snapshot))
    {
        return false;
    }

    host.with_global_mut(
        RunnerMonitorTopologyDiagnosticsStore::default,
        |store, _host| store.update_snapshot(snapshot),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        collections::HashMap,
    };

    use crate::ui_host::GlobalsHost;

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
        assert!(store.snapshot_matches(&snapshot));
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

    #[derive(Default)]
    struct TestGlobalsHost {
        globals: HashMap<TypeId, Box<dyn Any>>,
        tracked_mutations: usize,
    }

    impl GlobalsHost for TestGlobalsHost {
        fn set_global<T: Any>(&mut self, value: T) {
            self.globals.insert(TypeId::of::<T>(), Box::new(value));
            self.tracked_mutations += 1;
        }

        fn global<T: Any>(&self) -> Option<&T> {
            self.globals.get(&TypeId::of::<T>())?.downcast_ref::<T>()
        }

        fn with_global_mut<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = self
                .globals
                .remove(&type_id)
                .and_then(|value| value.downcast::<T>().ok().map(|value| *value))
                .unwrap_or_else(init);
            let result = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            self.tracked_mutations += 1;
            result
        }

        fn with_global_mut_untracked<T: Any, R>(
            &mut self,
            init: impl FnOnce() -> T,
            f: impl FnOnce(&mut T, &mut Self) -> R,
        ) -> R {
            let type_id = TypeId::of::<T>();
            let mut value = self
                .globals
                .remove(&type_id)
                .and_then(|value| value.downcast::<T>().ok().map(|value| *value))
                .unwrap_or_else(init);
            let result = f(&mut value, self);
            self.globals.insert(type_id, Box::new(value));
            result
        }
    }

    #[test]
    fn topology_update_marks_global_changed_only_for_real_snapshot_changes() {
        let mut host = TestGlobalsHost::default();
        let snapshot = RunnerMonitorTopologySnapshotV1 {
            virtual_desktop_bounds_physical: Some(rect(0, 0, 1920, 1080)),
            monitors: vec![RunnerMonitorInfoV1 {
                bounds_physical: rect(0, 0, 1920, 1080),
                scale_factor: 1.0,
            }],
        };

        assert!(update_runner_monitor_topology_diagnostics(
            &mut host,
            snapshot.clone()
        ));
        assert_eq!(host.tracked_mutations, 1);
        assert_eq!(
            host.global::<RunnerMonitorTopologyDiagnosticsStore>()
                .and_then(RunnerMonitorTopologyDiagnosticsStore::snapshot),
            Some(snapshot.clone())
        );

        assert!(!update_runner_monitor_topology_diagnostics(
            &mut host,
            snapshot.clone()
        ));
        assert_eq!(
            host.tracked_mutations, 1,
            "unchanged per-frame monitor topology refreshes must not produce global-change work"
        );

        let changed = RunnerMonitorTopologySnapshotV1 {
            virtual_desktop_bounds_physical: Some(rect(0, 0, 3840, 2160)),
            monitors: vec![RunnerMonitorInfoV1 {
                bounds_physical: rect(0, 0, 3840, 2160),
                scale_factor: 1.5,
            }],
        };
        assert!(update_runner_monitor_topology_diagnostics(
            &mut host, changed
        ));
        assert_eq!(host.tracked_mutations, 2);
    }
}
