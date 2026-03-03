use super::window::MonitorRectF64;
use winit::dpi::PhysicalPosition;

use std::ffi::c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct MonitorInfo {
    cb_size: u32,
    rc_monitor: Rect,
    rc_work: Rect,
    dw_flags: u32,
}

const MONITOR_DEFAULTTONEAREST: u32 = 2;

#[link(name = "user32")]
unsafe extern "system" {
    fn GetCursorPos(lpPoint: *mut Point) -> i32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
    fn MonitorFromPoint(pt: Point, dwFlags: u32) -> isize;
    fn GetMonitorInfoW(hMonitor: isize, lpmi: *mut MonitorInfo) -> i32;
    fn EnumWindows(
        lpEnumFunc: unsafe extern "system" fn(isize, isize) -> i32,
        lParam: isize,
    ) -> i32;
    fn WindowFromPoint(point: Point) -> isize;
    fn GetAncestor(hwnd: isize, ga_flags: u32) -> isize;
    fn GetWindow(hwnd: isize, cmd: u32) -> isize;
    fn GetWindowRect(hwnd: isize, lp_rect: *mut Rect) -> i32;
    fn ClientToScreen(hwnd: isize, lp_point: *mut Point) -> i32;
    fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    fn SetWindowLongW(hwnd: isize, index: i32, new_long: i32) -> i32;
    fn SetLayeredWindowAttributes(hwnd: isize, cr_key: u32, alpha: u8, flags: u32) -> i32;
    fn SetForegroundWindow(hwnd: isize) -> i32;
    fn SetWindowPos(
        hwnd: isize,
        hwnd_insert_after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
}

const VK_LBUTTON: i32 = 0x01;
const GA_ROOT: u32 = 2;
const GW_HWNDNEXT: u32 = 2;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_LAYERED: i32 = 0x0008_0000;
const WS_EX_TRANSPARENT: i32 = 0x0000_0020;
const LWA_ALPHA: u32 = 0x0000_0002;
const HWND_TOP: isize = 0;
const SWP_NOMOVE: u32 = 0x0002;
const SWP_NOSIZE: u32 = 0x0001;
const SWP_NOZORDER: u32 = 0x0004;
const SWP_SHOWWINDOW: u32 = 0x0040;

// DWM system backdrop (Windows 11 22H2+).
//
// We intentionally keep this best-effort: use the stable `DWMWA_SYSTEMBACKDROP_TYPE` attribute
// when available and clamp otherwise.
//
// References:
// - DWMWA_SYSTEMBACKDROP_TYPE (38)
// - DWM_SYSTEMBACKDROP_TYPE (AUTO/NONE/MAINWINDOW/TRANSIENTWINDOW/TABBEDWINDOW)
const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
const DWMSBT_AUTO: u32 = 0;
const DWMSBT_NONE: u32 = 1;
const DWMSBT_MAINWINDOW: u32 = 2;
const DWMSBT_TRANSIENTWINDOW: u32 = 3;

#[link(name = "dwmapi")]
unsafe extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const c_void,
        cb_attribute: u32,
    ) -> i32;
}

#[repr(C)]
#[derive(Clone, Copy)]
struct OsVersionInfoExW {
    dw_os_version_info_size: u32,
    dw_major_version: u32,
    dw_minor_version: u32,
    dw_build_number: u32,
    dw_platform_id: u32,
    sz_csd_version: [u16; 128],
    w_service_pack_major: u16,
    w_service_pack_minor: u16,
    w_suite_mask: u16,
    w_product_type: u8,
    w_reserved: u8,
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn RtlGetVersion(info: *mut OsVersionInfoExW) -> i32;
}

pub(super) fn windows_build_number() -> Option<u32> {
    let mut info = OsVersionInfoExW {
        dw_os_version_info_size: std::mem::size_of::<OsVersionInfoExW>() as u32,
        dw_major_version: 0,
        dw_minor_version: 0,
        dw_build_number: 0,
        dw_platform_id: 0,
        sz_csd_version: [0u16; 128],
        w_service_pack_major: 0,
        w_service_pack_minor: 0,
        w_suite_mask: 0,
        w_product_type: 0,
        w_reserved: 0,
    };
    // SAFETY: `info` is a valid, properly sized struct for the duration of the call.
    let status = unsafe { RtlGetVersion(&mut info) };
    if status != 0 {
        return None;
    }
    Some(info.dw_build_number)
}

pub(super) fn supports_dwm_system_backdrop() -> bool {
    // `DWMWA_SYSTEMBACKDROP_TYPE` is officially supported on Windows 11 22H2 (build 22621) and up.
    // We keep this conservative to avoid "capabilities lie".
    windows_build_number().is_some_and(|b| b >= 22621)
}

pub(super) fn set_dwm_system_backdrop_type(hwnd: isize, ty: u32) -> bool {
    if hwnd == 0 {
        return false;
    }
    if !supports_dwm_system_backdrop() {
        return false;
    }

    // SAFETY: `DwmSetWindowAttribute` reads `cb_attribute` bytes from `pv_attribute` synchronously.
    let hr = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &ty as *const u32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        )
    };
    hr >= 0
}

pub(super) fn dwm_system_backdrop_type_for_none() -> u32 {
    DWMSBT_NONE
}

pub(super) fn dwm_system_backdrop_type_for_system_default() -> u32 {
    DWMSBT_AUTO
}

pub(super) fn dwm_system_backdrop_type_for_mica() -> u32 {
    DWMSBT_MAINWINDOW
}

pub(super) fn dwm_system_backdrop_type_for_acrylic() -> u32 {
    DWMSBT_TRANSIENTWINDOW
}

pub(super) fn cursor_pos_physical() -> Option<PhysicalPosition<f64>> {
    let mut p = Point::default();
    let ok = unsafe { GetCursorPos(&mut p) };
    if ok == 0 {
        return None;
    }
    Some(PhysicalPosition::new(p.x as f64, p.y as f64))
}

pub(super) fn is_left_mouse_down() -> bool {
    // High-order bit is 1 when the key is down.
    // https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-getasynckeystate
    (unsafe { GetAsyncKeyState(VK_LBUTTON) } as i32 & 0x8000) != 0
}

pub(super) fn window_under_cursor_root(screen_pos: PhysicalPosition<f64>) -> Option<isize> {
    let pt = Point {
        x: screen_pos.x.round() as i32,
        y: screen_pos.y.round() as i32,
    };
    let hwnd = unsafe { WindowFromPoint(pt) };
    if hwnd == 0 {
        return None;
    }
    Some(root_hwnd(hwnd))
}

pub(super) fn next_window_in_z_order(hwnd: isize) -> Option<isize> {
    let next = unsafe { GetWindow(hwnd, GW_HWNDNEXT) };
    if next == 0 { None } else { Some(next) }
}

pub(super) fn root_hwnd(hwnd: isize) -> isize {
    if hwnd == 0 {
        return 0;
    }
    let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
    if root == 0 { hwnd } else { root }
}

pub(super) fn enum_windows_z_order() -> Vec<isize> {
    unsafe extern "system" fn callback(hwnd: isize, lparam: isize) -> i32 {
        if hwnd == 0 || lparam == 0 {
            return 1;
        }
        // SAFETY: `lparam` is a pointer to a live `Vec<isize>` owned by `enum_windows_z_order`.
        let out = unsafe { &mut *(lparam as *mut Vec<isize>) };
        out.push(hwnd);
        1
    }

    let mut out: Vec<isize> = Vec::new();
    // SAFETY: Win32 calls `callback` synchronously; we pass a valid pointer for the duration of
    // the call.
    unsafe {
        let _ = EnumWindows(callback, (&mut out as *mut Vec<isize>) as isize);
    }
    out
}

pub(super) fn screen_pos_in_hwnd(hwnd: isize, screen_pos: PhysicalPosition<f64>) -> bool {
    if hwnd == 0 {
        return false;
    }
    let mut rect = Rect::default();
    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    if ok == 0 {
        return false;
    }
    let x = screen_pos.x;
    let y = screen_pos.y;
    x >= rect.left as f64 && y >= rect.top as f64 && x < rect.right as f64 && y < rect.bottom as f64
}

pub(super) fn window_rect_screen_for_hwnd(hwnd: isize) -> Option<(i32, i32, i32, i32)> {
    if hwnd == 0 {
        return None;
    }
    let mut rect = Rect::default();
    let ok = unsafe { GetWindowRect(hwnd, &mut rect) };
    if ok == 0 {
        return None;
    }
    Some((rect.left, rect.top, rect.right, rect.bottom))
}

pub(super) fn decoration_offset_for_hwnd(hwnd: isize) -> Option<winit::dpi::PhysicalPosition<i32>> {
    if hwnd == 0 {
        return None;
    }

    let mut outer = Rect::default();
    let ok = unsafe { GetWindowRect(hwnd, &mut outer) };
    if ok == 0 {
        return None;
    }

    // Client origin in screen coordinates. This is the most robust way to recover "decoration
    // offset" (client origin relative to outer origin) on Windows, including under mixed DPI.
    let mut client = Point { x: 0, y: 0 };
    let ok = unsafe { ClientToScreen(hwnd, &mut client) };
    if ok == 0 {
        return None;
    }

    Some(winit::dpi::PhysicalPosition::new(
        client.x.saturating_sub(outer.left),
        client.y.saturating_sub(outer.top),
    ))
}

pub(super) fn client_origin_screen_for_hwnd(hwnd: isize) -> Option<PhysicalPosition<f64>> {
    if hwnd == 0 {
        return None;
    }
    let mut client = Point { x: 0, y: 0 };
    let ok = unsafe { ClientToScreen(hwnd, &mut client) };
    if ok == 0 {
        return None;
    }
    Some(PhysicalPosition::new(client.x as f64, client.y as f64))
}

pub(super) fn set_window_mouse_passthrough(hwnd: isize, enabled: bool) {
    if hwnd == 0 {
        return;
    }

    // NOTE: Many "click-through overlay" recipes rely on `WS_EX_TRANSPARENT` (often combined with
    // `WS_EX_LAYERED`). This is best-effort; some window styles/backends may still route mouse
    // events to the window depending on WM behavior. We keep this tightly scoped to dock tear-off.
    unsafe {
        let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if enabled {
            ex_style |= WS_EX_TRANSPARENT;
        } else {
            ex_style &= !WS_EX_TRANSPARENT;
        }
        let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);
    }
}

pub(super) fn set_window_alpha(hwnd: isize, alpha: f32) {
    if hwnd == 0 {
        return;
    }

    let a = alpha.clamp(0.0, 1.0);
    let byte_alpha = (255.0 * a).round().clamp(0.0, 255.0) as u8;

    unsafe {
        let mut ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
        if a < 1.0 {
            ex_style |= WS_EX_LAYERED;
            let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);
            let _ = SetLayeredWindowAttributes(hwnd, 0, byte_alpha, LWA_ALPHA);
        } else {
            // Restore to opaque. Keep it simple and remove layered to avoid staying in the
            // "layered window" path longer than needed.
            ex_style &= !WS_EX_LAYERED;
            let _ = SetWindowLongW(hwnd, GWL_EXSTYLE, ex_style);
        }
    }
}

pub(super) fn raise_hwnd_to_front(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }

    unsafe {
        // Best-effort: `SetWindowPos(HWND_TOP)` updates z-order without needing an activation
        // token. `SetForegroundWindow` is opportunistic; it may fail under focus-stealing rules,
        // but z-order is still expected to reflect the `SetWindowPos` call for in-process windows.
        let ok = SetWindowPos(
            hwnd,
            HWND_TOP,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
        ) != 0;
        let _ = SetForegroundWindow(hwnd);
        ok
    }
}

pub(super) fn set_window_outer_position(hwnd: isize, x: i32, y: i32) -> bool {
    if hwnd == 0 {
        return false;
    }

    unsafe {
        // Best-effort: move in *virtual desktop* physical coordinates without changing z-order.
        SetWindowPos(
            hwnd,
            0,
            x,
            y,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_SHOWWINDOW,
        ) != 0
    }
}

pub(super) fn monitor_work_area_for_point(point: PhysicalPosition<f64>) -> Option<MonitorRectF64> {
    let pt = Point {
        x: point.x.round() as i32,
        y: point.y.round() as i32,
    };
    let hmon = unsafe { MonitorFromPoint(pt, MONITOR_DEFAULTTONEAREST) };
    if hmon == 0 {
        return None;
    }

    let mut info = MonitorInfo {
        cb_size: std::mem::size_of::<MonitorInfo>() as u32,
        ..Default::default()
    };
    let ok = unsafe { GetMonitorInfoW(hmon, &mut info) };
    if ok == 0 {
        return None;
    }

    Some(MonitorRectF64 {
        min_x: info.rc_work.left as f64,
        min_y: info.rc_work.top as f64,
        max_x: info.rc_work.right as f64,
        max_y: info.rc_work.bottom as f64,
    })
}
