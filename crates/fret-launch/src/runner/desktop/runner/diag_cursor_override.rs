use std::path::PathBuf;
use std::time::SystemTime;

use fret_core::AppWindowId;
use slotmap::KeyData;
use winit::dpi::PhysicalPosition;
#[cfg(target_os = "windows")]
use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
#[cfg(target_os = "windows")]
use winit::window::Window;

#[derive(Debug)]
pub(super) struct DiagCursorScreenPosOverride {
    request_path: PathBuf,
    trigger_path: PathBuf,
    last_trigger_mtime: Option<SystemTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CursorOverrideKindV1 {
    ScreenPhysical,
    WindowClientPhysical,
    WindowClientLogical,
}

#[cfg(target_os = "windows")]
fn win32_decoration_offset_for_window(
    window: &dyn Window,
) -> Option<winit::dpi::PhysicalPosition<i32>> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    let hwnd = handle.hwnd.get();
    super::win32::decoration_offset_for_hwnd(hwnd)
}

impl DiagCursorScreenPosOverride {
    pub(super) fn from_env() -> Option<Self> {
        let out_dir_env = std::env::var_os("FRET_DIAG_DIR").filter(|v| !v.is_empty());
        let enabled =
            std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()) || out_dir_env.is_some();
        if !enabled {
            return None;
        }

        let out_dir = out_dir_env
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target").join("fret-diag"));

        Some(Self {
            request_path: out_dir.join("cursor_screen_pos.override.txt"),
            trigger_path: out_dir.join("cursor_screen_pos.touch"),
            last_trigger_mtime: None,
        })
    }

    fn poll<D: super::WinitAppDriver>(&mut self, runner: &mut super::WinitRunner<D>) -> bool {
        let modified = match std::fs::metadata(&self.trigger_path).and_then(|m| m.modified()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if self.last_trigger_mtime.is_some_and(|prev| prev >= modified) {
            return false;
        }
        self.last_trigger_mtime = Some(modified);

        let text = match std::fs::read_to_string(&self.request_path) {
            Ok(t) => t,
            Err(_) => return false,
        };

        let Some((kind, window_ffi, x_px, y_px)) = parse_cursor_override_v1(&text) else {
            return false;
        };

        let screen_pos = match kind {
            CursorOverrideKindV1::WindowClientPhysical => {
                let Some(window_ffi) = window_ffi else {
                    return false;
                };
                let window = AppWindowId::from(KeyData::from_ffi(window_ffi));
                let Some(state) = runner.windows.get(window) else {
                    return false;
                };
                let Ok(outer) = state.window.outer_position() else {
                    return false;
                };
                #[cfg(target_os = "windows")]
                let deco = win32_decoration_offset_for_window(state.window.as_ref())
                    .unwrap_or_else(|| state.window.surface_position());
                #[cfg(not(target_os = "windows"))]
                let deco = state.window.surface_position();
                let origin = super::window::client_origin_screen(outer, deco);
                PhysicalPosition::new(origin.x + x_px, origin.y + y_px)
            }
            CursorOverrideKindV1::WindowClientLogical => {
                let Some(window_ffi) = window_ffi else {
                    return false;
                };
                let window = AppWindowId::from(KeyData::from_ffi(window_ffi));
                let Some(state) = runner.windows.get(window) else {
                    return false;
                };
                let Ok(outer) = state.window.outer_position() else {
                    return false;
                };
                #[cfg(target_os = "windows")]
                let deco = win32_decoration_offset_for_window(state.window.as_ref())
                    .unwrap_or_else(|| state.window.surface_position());
                #[cfg(not(target_os = "windows"))]
                let deco = state.window.surface_position();
                let origin = super::window::client_origin_screen(outer, deco);
                let scale = state.window.scale_factor().max(0.000_001);
                PhysicalPosition::new(origin.x + (x_px * scale), origin.y + (y_px * scale))
            }
            CursorOverrideKindV1::ScreenPhysical => PhysicalPosition::new(x_px, y_px),
        };

        runner.cursor_screen_pos = Some(screen_pos);
        true
    }
}

fn parse_cursor_override_v1(text: &str) -> Option<(CursorOverrideKindV1, Option<u64>, f64, f64)> {
    let mut schema_version: Option<u32> = None;
    let mut kind: Option<CursorOverrideKindV1> = None;
    let mut window_ffi: Option<u64> = None;
    let mut x_px: Option<f64> = None;
    let mut y_px: Option<f64> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (k, v) = line.split_once('=')?;
        let k = k.trim();
        let v = v.trim();
        match k {
            "schema_version" => schema_version = v.parse().ok(),
            "kind" => {
                kind = match v {
                    "screen_physical" => Some(CursorOverrideKindV1::ScreenPhysical),
                    "window_client_physical" => Some(CursorOverrideKindV1::WindowClientPhysical),
                    "window_client_logical" => Some(CursorOverrideKindV1::WindowClientLogical),
                    _ => None,
                }
            }
            "window" => window_ffi = v.parse().ok(),
            "x_px" => x_px = v.parse().ok(),
            "y_px" => y_px = v.parse().ok(),
            _ => {}
        }
    }

    if schema_version != Some(1) {
        return None;
    }

    let kind = kind.unwrap_or(CursorOverrideKindV1::ScreenPhysical);
    Some((kind, window_ffi, x_px?, y_px?))
}

impl<D: super::WinitAppDriver> super::WinitRunner<D> {
    pub(super) fn poll_diag_cursor_screen_pos_override(&mut self) -> bool {
        let Some(mut svc) = self.diag_cursor_screen_pos_override.take() else {
            return false;
        };
        let updated = svc.poll(self);
        self.diag_cursor_screen_pos_override = Some(svc);
        updated
    }
}
