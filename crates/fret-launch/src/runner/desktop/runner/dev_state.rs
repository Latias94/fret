use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use winit::dpi::{LogicalSize, PhysicalPosition, Position};

use fret_app::App;

use super::WindowCreateSpec;

use crate::dev_state::DevStateService;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestoreOutcome {
    Disabled,
    ResetRequested,
    FileNotFound,
    ParseFailed,
    Restored,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct MainWindowGeometry {
    logical_size: LogicalSize<f64>,
    position: Option<PhysicalPosition<i32>>,
}

#[derive(Debug)]
pub(crate) struct DevStateController {
    enabled: bool,
    path: PathBuf,
    poll_interval: Duration,
    debounce: Duration,

    restore_outcome: RestoreOutcome,
    restored_main: Option<MainWindowGeometry>,
    restore_logged: bool,
    restore_printed: bool,
    incoming_app: std::collections::HashMap<String, serde_json::Value>,

    next_poll_at: Instant,
    last_observed: Option<MainWindowGeometry>,
    dirty_since: Option<Instant>,
    last_app_epoch: u64,
}

impl DevStateController {
    pub(crate) fn from_env(now: Instant) -> Self {
        let enabled = cfg!(debug_assertions)
            && (std::env::var_os("FRET_DEV_STATE").is_some_and(|v| !v.is_empty())
                || std::env::var_os("FRET_HOTPATCH").is_some_and(|v| !v.is_empty())
                || std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty()));

        let path = std::env::var_os("FRET_DEV_STATE_PATH")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(".fret").join("dev_state.json"));

        let poll_interval = std::env::var("FRET_DEV_STATE_POLL_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(150));

        let debounce = std::env::var("FRET_DEV_STATE_DEBOUNCE_MS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(350));

        let reset_requested =
            std::env::var_os("FRET_DEV_STATE_RESET").is_some_and(|v| !v.is_empty());

        let (restore_outcome, restored_main, incoming_app) = if !enabled {
            (
                RestoreOutcome::Disabled,
                None,
                std::collections::HashMap::new(),
            )
        } else if reset_requested {
            let _ = std::fs::remove_file(&path);
            (
                RestoreOutcome::ResetRequested,
                None,
                std::collections::HashMap::new(),
            )
        } else {
            match load_dev_state_file(&path) {
                Ok(Some(file)) => {
                    let geom = file.windows.get("main").map(|geom| MainWindowGeometry {
                        logical_size: LogicalSize::new(
                            geom.logical_size.width,
                            geom.logical_size.height,
                        ),
                        position: geom.position.map(|p| PhysicalPosition::new(p.x, p.y)),
                    });
                    (RestoreOutcome::Restored, geom, file.app)
                }
                Ok(None) => (
                    RestoreOutcome::FileNotFound,
                    None,
                    std::collections::HashMap::new(),
                ),
                Err(_) => (
                    RestoreOutcome::ParseFailed,
                    None,
                    std::collections::HashMap::new(),
                ),
            }
        };

        Self {
            enabled,
            path,
            poll_interval,
            debounce,
            restore_outcome,
            restored_main,
            restore_logged: false,
            restore_printed: false,
            incoming_app,
            next_poll_at: now + poll_interval,
            last_observed: None,
            dirty_since: None,
            last_app_epoch: 0,
        }
    }

    pub(crate) fn install_into_app(&mut self, app: &mut App) {
        if !self.enabled {
            return;
        }
        let incoming = std::mem::take(&mut self.incoming_app);
        app.set_global(DevStateService::new(incoming, self.path.clone()));
    }

    pub(crate) fn enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn apply_main_window_spec(&mut self, spec: &mut WindowCreateSpec) {
        if !self.enabled {
            return;
        }

        if let Some(geom) = self.restored_main {
            spec.size = geom.logical_size;
            if let Some(pos) = geom.position {
                spec.position = Some(Position::Physical(pos));
            }
        }

        if !self.restore_logged {
            self.restore_logged = true;
            match self.restore_outcome {
                RestoreOutcome::Disabled => {}
                RestoreOutcome::ResetRequested => {
                    info!(
                        path = %self.path.display(),
                        "dev_state: reset requested (restore skipped)"
                    );
                }
                RestoreOutcome::FileNotFound => {
                    debug!(
                        path = %self.path.display(),
                        "dev_state: no dev state file (restore skipped)"
                    );
                }
                RestoreOutcome::ParseFailed => {
                    warn!(
                        path = %self.path.display(),
                        "dev_state: failed to parse dev state (restore skipped)"
                    );
                }
                RestoreOutcome::Restored => {
                    let restored_position = self.restored_main.and_then(|g| g.position).is_some();
                    info!(
                        path = %self.path.display(),
                        size = ?self.restored_main.map(|g| (g.logical_size.width, g.logical_size.height)),
                        restored_position = restored_position,
                        "dev_state: restored main window geometry",
                    );
                }
            }
        }

        if !self.restore_printed {
            self.restore_printed = true;
            match self.restore_outcome {
                RestoreOutcome::Disabled => {}
                RestoreOutcome::ResetRequested => {
                    eprintln!(
                        "dev_state: reset requested (restore skipped): {}",
                        self.path.display()
                    );
                }
                RestoreOutcome::FileNotFound => {
                    eprintln!(
                        "dev_state: file not found (restore skipped): {}",
                        self.path.display()
                    );
                }
                RestoreOutcome::ParseFailed => {
                    eprintln!(
                        "dev_state: parse failed (restore skipped): {}",
                        self.path.display()
                    );
                }
                RestoreOutcome::Restored => {
                    let restored_position = self.restored_main.and_then(|g| g.position).is_some();
                    eprintln!(
                        "dev_state: restored main window (restored_position={restored_position}) from {}",
                        self.path.display()
                    );
                }
            }
        }
    }

    pub(crate) fn observe_main_window(
        &mut self,
        now: Instant,
        app: &App,
        logical_size: LogicalSize<f64>,
        position: Option<PhysicalPosition<i32>>,
    ) {
        if !self.enabled {
            return;
        }

        if now < self.next_poll_at {
            return;
        }
        self.next_poll_at = now + self.poll_interval;

        let observed = MainWindowGeometry {
            logical_size,
            position,
        };

        let app_epoch = app
            .global::<DevStateService>()
            .map(|svc| svc.outgoing_snapshot().epoch)
            .unwrap_or(0);

        if self.last_observed.is_some_and(|prev| prev == observed)
            && app_epoch == self.last_app_epoch
        {
            if let Some(since) = self.dirty_since
                && now.saturating_duration_since(since) >= self.debounce
            {
                if let Err(err) = self.flush_main_window(app, observed) {
                    warn!(path = %self.path.display(), error = %err, "dev_state: flush failed");
                }
                self.dirty_since = None;
            }
            return;
        }

        self.last_observed = Some(observed);
        self.last_app_epoch = app_epoch;
        self.dirty_since = Some(now);
    }

    fn flush_main_window(&mut self, app: &App, geom: MainWindowGeometry) -> Result<(), String> {
        let app_data = app
            .global::<DevStateService>()
            .map(|svc| svc.outgoing_snapshot().data)
            .unwrap_or_default();

        let file = DevStateFileV1 {
            version: 1,
            updated_at_ms: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or_default(),
            app: app_data,
            windows: std::collections::HashMap::from([(
                "main".to_string(),
                WindowGeometryV1 {
                    logical_size: LogicalSizeV1 {
                        width: geom.logical_size.width,
                        height: geom.logical_size.height,
                    },
                    position: geom.position.map(|p| PhysicalPositionV1 { x: p.x, y: p.y }),
                },
            )]),
        };

        write_json_atomic(&self.path, &file)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DevStateFileV1 {
    version: u32,
    #[serde(default)]
    updated_at_ms: u64,
    #[serde(default)]
    app: std::collections::HashMap<String, serde_json::Value>,
    #[serde(default)]
    windows: std::collections::HashMap<String, WindowGeometryV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct WindowGeometryV1 {
    logical_size: LogicalSizeV1,
    #[serde(default)]
    position: Option<PhysicalPositionV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct LogicalSizeV1 {
    width: f64,
    height: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct PhysicalPositionV1 {
    x: i32,
    y: i32,
}

fn load_dev_state_file(path: &Path) -> Result<Option<DevStateFileV1>, String> {
    let bytes = match std::fs::read(path) {
        Ok(v) => v,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err.to_string()),
    };

    let file: DevStateFileV1 = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if file.version != 1 {
        return Ok(None);
    }
    Ok(Some(file))
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let Some(dir) = path.parent() else {
        return Err("missing parent directory".to_string());
    };
    if !dir.as_os_str().is_empty() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }

    let tmp = tmp_path_for(path);
    let data = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, data).map_err(|e| e.to_string())?;

    replace_file_atomic(&tmp, path)?;
    Ok(())
}

fn tmp_path_for(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("dev_state.json");
    let tmp_name = format!("{file_name}.{pid}.tmp");
    path.with_file_name(tmp_name)
}

#[cfg(not(windows))]
fn replace_file_atomic(tmp: &Path, dest: &Path) -> Result<(), String> {
    std::fs::rename(tmp, dest).map_err(|e| e.to_string())
}

#[cfg(windows)]
fn replace_file_atomic(tmp: &Path, dest: &Path) -> Result<(), String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt as _;
    use windows_sys::Win32::Storage::FileSystem::{
        MOVEFILE_REPLACE_EXISTING, MoveFileExW, ReplaceFileW,
    };

    fn to_wide(s: &OsStr) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_wide().collect();
        v.push(0);
        v
    }

    let dest_exists = dest.is_file();

    let tmp_w = to_wide(tmp.as_os_str());
    let dest_w = to_wide(dest.as_os_str());

    unsafe {
        if dest_exists {
            let ok = ReplaceFileW(
                dest_w.as_ptr(),
                tmp_w.as_ptr(),
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if ok != 0 {
                return Ok(());
            }
        }

        let ok = MoveFileExW(tmp_w.as_ptr(), dest_w.as_ptr(), MOVEFILE_REPLACE_EXISTING);
        if ok != 0 {
            return Ok(());
        }
    }

    Err("failed to atomically replace dev-state file".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v1_main_window_geometry() {
        let json = r#"
        {
          "version": 1,
          "updated_at_ms": 123,
          "windows": {
            "main": {
              "logical_size": { "width": 1000.0, "height": 700.0 },
              "position": { "x": 10, "y": 20 }
            }
          }
        }"#;

        let tmp = std::env::temp_dir().join("fret_dev_state_parse_test.json");
        let _ = std::fs::write(&tmp, json);

        let file = load_dev_state_file(&tmp).expect("parse").expect("present");
        let geom = file.windows.get("main").copied().expect("main");
        assert_eq!(geom.logical_size.width, 1000.0);
        assert_eq!(geom.logical_size.height, 700.0);
        assert_eq!(geom.position.map(|p| (p.x, p.y)), Some((10, 20)));
    }

    #[test]
    fn ignores_unknown_version() {
        let json = r#"{ "version": 999, "windows": { "main": { "logical_size": { "width": 1.0, "height": 2.0 } } } }"#;
        let tmp = std::env::temp_dir().join("fret_dev_state_parse_test_unknown_version.json");
        let _ = std::fs::write(&tmp, json);
        let file = load_dev_state_file(&tmp).expect("read");
        assert!(file.is_none());
    }

    #[test]
    fn preserves_app_map() {
        let json = r#"
        {
          "version": 1,
          "app": { "docking.layout": { "layout_version": 2, "windows": [], "nodes": [] } },
          "windows": {
            "main": { "logical_size": { "width": 1.0, "height": 2.0 } }
          }
        }"#;
        let tmp = std::env::temp_dir().join("fret_dev_state_parse_test_app.json");
        let _ = std::fs::write(&tmp, json);
        let file = load_dev_state_file(&tmp).expect("read").expect("present");
        assert!(file.app.contains_key("docking.layout"));
    }
}
