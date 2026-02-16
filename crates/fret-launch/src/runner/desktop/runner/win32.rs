use super::window::MonitorRectF64;
use winit::dpi::PhysicalPosition;

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
    fn WindowFromPoint(point: Point) -> isize;
    fn GetAncestor(hwnd: isize, ga_flags: u32) -> isize;
    fn GetWindow(hwnd: isize, cmd: u32) -> isize;
    fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    fn SetWindowLongW(hwnd: isize, index: i32, new_long: i32) -> i32;
    fn SetLayeredWindowAttributes(hwnd: isize, cr_key: u32, alpha: u8, flags: u32) -> i32;
}

const VK_LBUTTON: i32 = 0x01;
const GA_ROOT: u32 = 2;
const GW_HWNDNEXT: u32 = 2;
const GWL_EXSTYLE: i32 = -20;
const WS_EX_LAYERED: i32 = 0x0008_0000;
const WS_EX_TRANSPARENT: i32 = 0x0000_0020;
const LWA_ALPHA: u32 = 0x0000_0002;

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
    let root = unsafe { GetAncestor(hwnd, GA_ROOT) };
    if root == 0 { Some(hwnd) } else { Some(root) }
}

pub(super) fn next_window_in_z_order(hwnd: isize) -> Option<isize> {
    let next = unsafe { GetWindow(hwnd, GW_HWNDNEXT) };
    if next == 0 { None } else { Some(next) }
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
