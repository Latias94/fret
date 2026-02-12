#![cfg(target_os = "windows")]

use super::MonitorRectF64;
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
    fn MonitorFromPoint(pt: Point, dwFlags: u32) -> isize;
    fn GetMonitorInfoW(hMonitor: isize, lpmi: *mut MonitorInfo) -> i32;
}

pub(super) fn cursor_pos_physical() -> Option<PhysicalPosition<f64>> {
    let mut p = Point::default();
    let ok = unsafe { GetCursorPos(&mut p) };
    if ok == 0 {
        return None;
    }
    Some(PhysicalPosition::new(p.x as f64, p.y as f64))
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
