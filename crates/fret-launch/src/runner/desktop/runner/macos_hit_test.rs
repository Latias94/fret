#![cfg(target_os = "macos")]

use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use fret_runtime::WindowHitTestRegionV1;

use objc2::MainThreadMarker;
use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSEvent, NSEventMask};
use objc2_foundation::{NSPoint, NSRect};
use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
use winit::window::Window;

use super::{EventLoopProxy, RunnerUserEvent};

static EVENT_LOOP_PROXY: OnceLock<EventLoopProxy> = OnceLock::new();
static PROXY_EVENTS: OnceLock<std::sync::Arc<std::sync::Mutex<Vec<RunnerUserEvent>>>> =
    OnceLock::new();

static COALESCED_REFRESH_PENDING: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
struct MacosHitTestRegionsState {
    by_ns_window_ptr: HashMap<isize, WindowHitTestRegionsWindowState>,
    global_mouse_monitor: Option<Retained<AnyObject>>,
}

struct WindowHitTestRegionsWindowState {
    ns_window: *mut AnyObject,
    regions: Box<[WindowHitTestRegionV1]>,
    last_ignores_mouse_events: Option<bool>,
}

pub(crate) fn set_event_loop_proxy(
    proxy: EventLoopProxy,
    events: std::sync::Arc<std::sync::Mutex<Vec<RunnerUserEvent>>>,
) {
    let _ = EVENT_LOOP_PROXY.set(proxy);
    let _ = PROXY_EVENTS.set(events);
}

thread_local! {
    static STATE: RefCell<MacosHitTestRegionsState> =
        RefCell::new(MacosHitTestRegionsState::default());
}

fn ns_window_id(window: &dyn Window) -> Option<*mut AnyObject> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::AppKit(h) = handle.as_raw() else {
        return None;
    };
    let ns_view = h.ns_view.as_ptr().cast::<AnyObject>();
    if ns_view.is_null() {
        return None;
    }
    unsafe {
        let ns_window: *mut AnyObject = msg_send![ns_view, window];
        (!ns_window.is_null()).then_some(ns_window)
    }
}

fn regions_contain_point(regions: &[WindowHitTestRegionV1], px: f32, py: f32) -> bool {
    fn rect_contains(x: f32, y: f32, w: f32, h: f32, px: f32, py: f32) -> bool {
        let x = if x.is_finite() { x } else { 0.0 };
        let y = if y.is_finite() { y } else { 0.0 };
        let w = if w.is_finite() { w.max(0.0) } else { 0.0 };
        let h = if h.is_finite() { h.max(0.0) } else { 0.0 };
        if w <= 0.0 || h <= 0.0 {
            return false;
        }
        px >= x && px < x + w && py >= y && py < y + h
    }

    fn rrect_contains(x: f32, y: f32, w: f32, h: f32, r: f32, px: f32, py: f32) -> bool {
        if !rect_contains(x, y, w, h, px, py) {
            return false;
        }
        let r = if r.is_finite() { r.max(0.0) } else { 0.0 };
        if r <= 0.0 {
            return true;
        }

        let max_r = 0.5 * w.min(h);
        let r = r.min(max_r);

        let left = x + r;
        let right = x + w - r;
        let top = y + r;
        let bottom = y + h - r;

        // Fast path: inside center cross.
        if (px >= left && px < right) || (py >= top && py < bottom) {
            return true;
        }

        // Corner circle tests.
        let cx = if px < left { left } else { right };
        let cy = if py < top { top } else { bottom };
        let dx = px - cx;
        let dy = py - cy;
        dx * dx + dy * dy <= r * r
    }

    for r in regions {
        match *r {
            WindowHitTestRegionV1::Rect {
                x,
                y,
                width,
                height,
            } => {
                if rect_contains(x, y, width, height, px, py) {
                    return true;
                }
            }
            WindowHitTestRegionV1::RRect {
                x,
                y,
                width,
                height,
                radius,
            } => {
                if rrect_contains(x, y, width, height, radius, px, py) {
                    return true;
                }
            }
        }
    }
    false
}

fn set_ignores_mouse_events(ns_window: *mut AnyObject, ignores: bool) -> bool {
    if ns_window.is_null() {
        return false;
    }
    unsafe {
        let _: () = msg_send![ns_window, setIgnoresMouseEvents: ignores];
    }
    true
}

fn apply_for_window_at_mouse_location(
    ws: &mut WindowHitTestRegionsWindowState,
    mouse_location_screen: NSPoint,
) {
    if ws.ns_window.is_null() {
        return;
    }

    // `mouseLocation` is in screen coordinates; `convertPointFromScreen:` yields window base
    // coordinates (origin at bottom-left). Our regions use client coordinates with (0,0) at the
    // top-left (ADR 0313), so we flip Y using the content view height.
    let local: NSPoint =
        unsafe { msg_send![ws.ns_window, convertPointFromScreen: mouse_location_screen] };
    let content_view: *mut AnyObject = unsafe { msg_send![ws.ns_window, contentView] };
    if content_view.is_null() {
        return;
    }
    let bounds: NSRect = unsafe { msg_send![content_view, bounds] };
    let content_h = bounds.size.height as f32;

    let px = local.x as f32;
    let py = content_h - local.y as f32;

    let interactive = regions_contain_point(&ws.regions, px, py);
    let ignores = !interactive;
    if ws.last_ignores_mouse_events == Some(ignores) {
        return;
    }
    if set_ignores_mouse_events(ws.ns_window, ignores) {
        ws.last_ignores_mouse_events = Some(ignores);
    }
}

fn install_global_mouse_monitor_if_needed(state: &mut MacosHitTestRegionsState) {
    if state.global_mouse_monitor.is_some() {
        return;
    }
    if state.by_ns_window_ptr.is_empty() {
        return;
    }

    // The global monitor handler may run off the main thread, so it must not touch AppKit
    // objects. We coalesce into a single runner wake-up and sample `NSEvent::mouseLocation()`
    // on the main thread instead.
    block2::global_block! {
        static MOUSE_MONITOR = |event: std::ptr::NonNull<NSEvent>| {
            let _ = event;
            if COALESCED_REFRESH_PENDING.swap(true, Ordering::SeqCst) {
                return;
            }
            let Some(events) = PROXY_EVENTS.get() else {
                COALESCED_REFRESH_PENDING.store(false, Ordering::SeqCst);
                return;
            };
            let Some(proxy) = EVENT_LOOP_PROXY.get() else {
                COALESCED_REFRESH_PENDING.store(false, Ordering::SeqCst);
                return;
            };
            if let Ok(mut queue) = events.lock() {
                queue.push(RunnerUserEvent::MacosHitTestRefreshRegions);
            }
            proxy.wake_up();
        };
    }

    let mask = NSEventMask::MouseMoved
        | NSEventMask::LeftMouseDragged
        | NSEventMask::RightMouseDragged
        | NSEventMask::OtherMouseDragged;

    let monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(mask, &MOUSE_MONITOR);
    state.global_mouse_monitor = monitor;
}

fn uninstall_global_mouse_monitor_if_needed(state: &mut MacosHitTestRegionsState) {
    if !state.by_ns_window_ptr.is_empty() {
        return;
    }
    let Some(monitor) = state.global_mouse_monitor.take() else {
        return;
    };
    // SAFETY: `monitor` is the object returned from AppKit.
    unsafe { NSEvent::removeMonitor(&monitor) };
}

pub(crate) fn set_passthrough_regions(
    window: &dyn Window,
    regions: &[WindowHitTestRegionV1],
) -> bool {
    let Some(ns_window) = ns_window_id(window) else {
        return false;
    };
    let key = ns_window as isize;

    // Conservative default: passthrough unless proven interactive at the current cursor point.
    let _ = set_ignores_mouse_events(ns_window, true);

    STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.by_ns_window_ptr.insert(
            key,
            WindowHitTestRegionsWindowState {
                ns_window,
                regions: regions.to_vec().into_boxed_slice(),
                last_ignores_mouse_events: Some(true),
            },
        );
        install_global_mouse_monitor_if_needed(&mut state);

        if MainThreadMarker::new().is_some() {
            let mouse_location = NSEvent::mouseLocation();
            if let Some(ws) = state.by_ns_window_ptr.get_mut(&key) {
                apply_for_window_at_mouse_location(ws, mouse_location);
            }
        }
    });

    true
}

pub(crate) fn clear_passthrough_regions(window: &dyn Window) {
    let Some(ns_window) = ns_window_id(window) else {
        return;
    };
    let key = ns_window as isize;

    STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        state.by_ns_window_ptr.remove(&key);
        uninstall_global_mouse_monitor_if_needed(&mut state);
    });
}

pub(crate) fn unregister_window(window: &dyn Window) {
    clear_passthrough_regions(window);
}

pub(crate) fn has_active_regions() -> bool {
    STATE.with(|state| {
        state
            .try_borrow()
            .map(|s| !s.by_ns_window_ptr.is_empty())
            .unwrap_or(false)
    })
}

pub(crate) fn apply_latest_mouse_location() {
    COALESCED_REFRESH_PENDING.store(false, Ordering::SeqCst);

    if MainThreadMarker::new().is_none() {
        return;
    };
    let mouse_location = NSEvent::mouseLocation();

    STATE.with(|state| {
        let Ok(mut state) = state.try_borrow_mut() else {
            return;
        };
        for ws in state.by_ns_window_ptr.values_mut() {
            apply_for_window_at_mouse_location(ws, mouse_location);
        }
    });
}
