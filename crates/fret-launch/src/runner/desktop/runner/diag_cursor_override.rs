use std::path::PathBuf;

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
    last_trigger_stamp: Option<u64>,
    last_window: Option<AppWindowId>,
    last_kind: Option<CursorOverrideKindV1>,
    last_local_px: Option<(f64, f64)>,
    last_screen_pos: Option<PhysicalPosition<f64>>,
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
    let hwnd = super::win32::root_hwnd(handle.hwnd.get());
    super::win32::decoration_offset_for_hwnd(hwnd)
}

#[cfg(target_os = "windows")]
fn win32_client_origin_screen_for_window(window: &dyn Window) -> Option<PhysicalPosition<f64>> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    let hwnd = super::win32::root_hwnd(handle.hwnd.get());
    super::win32::client_origin_screen_for_hwnd(hwnd)
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
            last_trigger_stamp: None,
            last_window: None,
            last_kind: None,
            last_local_px: None,
            last_screen_pos: None,
        })
    }

    fn poll<D: super::WinitAppDriver>(&mut self, runner: &mut super::WinitRunner<D>) -> bool {
        let stamp = match std::fs::read_to_string(&self.trigger_path) {
            Ok(text) => text
                .lines()
                .rev()
                .find_map(|line| line.trim().parse::<u64>().ok()),
            Err(_) => None,
        };
        let Some(stamp) = stamp else {
            return false;
        };
        if self.last_trigger_stamp.is_some_and(|prev| prev >= stamp) {
            return false;
        }
        self.last_trigger_stamp = Some(stamp);

        let text = match std::fs::read_to_string(&self.request_path) {
            Ok(t) => t,
            Err(_) => return false,
        };

        let Some((kind, window_ffi, x_px, y_px)) = parse_cursor_override_v1(&text) else {
            return false;
        };

        let screen_pos = match kind {
            CursorOverrideKindV1::ScreenPhysical => {
                self.last_window = None;
                self.last_kind = Some(kind);
                self.last_local_px = None;
                let p = PhysicalPosition::new(x_px, y_px);
                self.last_screen_pos = Some(p);
                p
            }
            CursorOverrideKindV1::WindowClientPhysical
            | CursorOverrideKindV1::WindowClientLogical => {
                let Some(window_ffi) = window_ffi else {
                    return false;
                };
                let window = AppWindowId::from(KeyData::from_ffi(window_ffi));
                let Some(state) = runner.windows.get(window) else {
                    return false;
                };
                #[cfg(target_os = "windows")]
                let origin = win32_client_origin_screen_for_window(state.window.as_ref())
                    .or_else(|| {
                        let outer = state.window.outer_position().ok()?;
                        let deco = win32_decoration_offset_for_window(state.window.as_ref())
                            .unwrap_or_else(|| state.window.surface_position());
                        Some(super::window::client_origin_screen(outer, deco))
                    })
                    .map(|p| (p.x, p.y));
                #[cfg(not(target_os = "windows"))]
                let origin = (|| {
                    let outer = state.window.outer_position().ok()?;
                    let deco = state.window.surface_position();
                    Some(super::window::client_origin_screen(outer, deco)).map(|p| (p.x, p.y))
                })();
                let Some((origin_x, origin_y)) = origin else {
                    return false;
                };

                // Diagnostics scripts typically inject pointer events in window-client coordinates.
                // When the runner also moves OS windows (tear-off follow), using the *current* window
                // origin would incorrectly drag the simulated cursor along with the moving window.
                //
                // To better approximate an OS cursor in screen space, treat consecutive window-client
                // overrides as *relative motion* deltas and integrate them into the previous screen
                // position when possible.
                let can_integrate_base =
                    self.last_window == Some(window) && self.last_kind == Some(kind);

                let (dx, dy) = if let Some((last_x, last_y)) = self.last_local_px
                    && can_integrate_base
                {
                    (x_px - last_x, y_px - last_y)
                } else {
                    (0.0, 0.0)
                };

                // Heuristic: only integrate small, stepwise updates. Large jumps are typically
                // "absolute" cursor placements (e.g. pointer-down targeting a node) and should
                // snap to the window's current origin.
                let max_delta_px = 256.0;
                let can_integrate =
                    can_integrate_base && dx.abs() <= max_delta_px && dy.abs() <= max_delta_px;

                let (dx_screen, dy_screen) = match kind {
                    CursorOverrideKindV1::WindowClientPhysical => (dx, dy),
                    CursorOverrideKindV1::WindowClientLogical => {
                        let scale = state.window.scale_factor().max(0.000_001);
                        (dx * scale, dy * scale)
                    }
                    CursorOverrideKindV1::ScreenPhysical => unreachable!(),
                };

                let pos = if can_integrate
                    && let Some(prev) = self.last_screen_pos
                    && self.last_local_px.is_some()
                {
                    PhysicalPosition::new(prev.x + dx_screen, prev.y + dy_screen)
                } else {
                    match kind {
                        CursorOverrideKindV1::WindowClientPhysical => {
                            PhysicalPosition::new(origin_x + x_px, origin_y + y_px)
                        }
                        CursorOverrideKindV1::WindowClientLogical => {
                            let scale = state.window.scale_factor().max(0.000_001);
                            PhysicalPosition::new(
                                origin_x + (x_px * scale),
                                origin_y + (y_px * scale),
                            )
                        }
                        CursorOverrideKindV1::ScreenPhysical => unreachable!(),
                    }
                };

                self.last_window = Some(window);
                self.last_kind = Some(kind);
                self.last_local_px = Some((x_px, y_px));
                self.last_screen_pos = Some(pos);
                pos
            }
        };

        runner.cursor_screen_pos = Some(screen_pos);
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        runner.refresh_platform_window_receiver_at_cursor_diagnostics();
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
        if updated {
            self.diag_last_cursor_override_tick = Some(self.tick_id);
        }
        updated
    }
}
