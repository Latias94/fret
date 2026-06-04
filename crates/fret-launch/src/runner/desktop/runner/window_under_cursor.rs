#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::collections::HashMap;

#[cfg(target_os = "macos")]
use fret_core::time::Instant;
use winit::{dpi::PhysicalPosition, window::Window};

#[cfg(target_os = "windows")]
use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

use super::{WinitAppDriver, WinitRunner};

#[derive(Debug, Clone, Copy)]
pub(super) struct WindowUnderCursorHit {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) source: fret_runtime::WindowUnderCursorSource,
}

impl<D: WinitAppDriver> WinitRunner<D> {
    #[cfg(target_os = "macos")]
    pub(super) fn ns_window_number_for_window(window: &dyn Window) -> Option<i32> {
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};
        use winit::raw_window_handle::HasWindowHandle as _;

        let ns_window: *mut Object = match window.window_handle() {
            Ok(handle) => match handle.as_raw() {
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                    let ns_view: *mut Object = h.ns_view.as_ptr() as *mut Object;
                    if ns_view.is_null() {
                        std::ptr::null_mut()
                    } else {
                        unsafe { msg_send![ns_view, window] }
                    }
                }
                _ => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        };
        if ns_window.is_null() {
            return None;
        }

        let number: i32 = unsafe { msg_send![ns_window, windowNumber] };
        Some(number)
    }

    #[cfg(target_os = "macos")]
    pub(super) fn ordered_ns_window_numbers_front_to_back() -> Vec<i32> {
        use objc::runtime::Class;
        use objc::runtime::Object;
        use objc::{msg_send, sel, sel_impl};

        unsafe {
            let Some(class) = Class::get("NSApplication") else {
                return Vec::new();
            };
            let app: *mut Object = msg_send![class, sharedApplication];
            if app.is_null() {
                return Vec::new();
            }
            let ordered: *mut Object = msg_send![app, orderedWindows];
            if ordered.is_null() {
                return Vec::new();
            }

            let count: usize = msg_send![ordered, count];
            let mut out: Vec<i32> = Vec::with_capacity(count);
            for idx in 0..count {
                let w: *mut Object = msg_send![ordered, objectAtIndex: idx];
                if w.is_null() {
                    continue;
                }
                let number: i32 = msg_send![w, windowNumber];
                out.push(number);
            }
            out
        }
    }

    #[cfg(target_os = "macos")]
    fn window_under_cursor_macos(
        &self,
        screen_pos: PhysicalPosition<f64>,
        prefer_not: Option<fret_core::AppWindowId>,
    ) -> Option<fret_core::AppWindowId> {
        let mut number_to_window: HashMap<i32, fret_core::AppWindowId> = HashMap::new();
        for (window, state) in self.windows.iter() {
            let Some(number) = Self::ns_window_number_for_window(state.window.as_ref()) else {
                continue;
            };
            number_to_window.insert(number, window);
        }

        if number_to_window.is_empty() {
            return None;
        }

        let prefer_not_number = prefer_not
            .and_then(|w| self.windows.get(w))
            .and_then(|state| Self::ns_window_number_for_window(state.window.as_ref()));

        let ordered = Self::ordered_ns_window_numbers_front_to_back();
        if ordered.is_empty() {
            return None;
        }

        let mut fallback: Option<fret_core::AppWindowId> = None;
        for number in ordered {
            let Some(&window) = number_to_window.get(&number) else {
                continue;
            };
            if !self.screen_pos_in_window(window, screen_pos) {
                continue;
            }
            if prefer_not_number.is_some_and(|p| p == number) {
                fallback = Some(window);
                continue;
            }
            return Some(window);
        }

        fallback
    }

    #[cfg(target_os = "windows")]
    pub(super) fn hwnd_for_window(window: &dyn Window) -> Option<isize> {
        let handle = window.window_handle().ok()?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return None;
        };
        Some(super::win32::root_hwnd(handle.hwnd.get()))
    }

    #[cfg(target_os = "windows")]
    fn window_under_cursor_win32(
        &self,
        screen_pos: PhysicalPosition<f64>,
        prefer_not: Option<fret_core::AppWindowId>,
    ) -> Option<fret_core::AppWindowId> {
        let mut hwnd_to_window: HashMap<isize, fret_core::AppWindowId> = HashMap::new();
        for (window, state) in self.windows.iter() {
            let Some(hwnd) = Self::hwnd_for_window(state.window.as_ref()) else {
                continue;
            };
            hwnd_to_window.insert(hwnd, window);
        }

        if hwnd_to_window.is_empty() {
            return None;
        }

        let prefer_not_hwnd = prefer_not
            .and_then(|w| self.windows.get(w))
            .and_then(|state| Self::hwnd_for_window(state.window.as_ref()));

        let mut fallback: Option<fret_core::AppWindowId> = None;
        let mut hwnd = super::win32::window_under_cursor_root(screen_pos)?;
        // Bounded traversal: the global z-order can change while we walk it.
        for _ in 0..256 {
            if hwnd == 0 {
                break;
            }

            if prefer_not_hwnd.is_some_and(|p| p == hwnd) {
                if let Some(&window) = hwnd_to_window.get(&hwnd)
                    && super::win32::screen_pos_in_hwnd(hwnd, screen_pos)
                {
                    fallback = Some(window);
                }
            } else if let Some(&window) = hwnd_to_window.get(&hwnd)
                && super::win32::screen_pos_in_hwnd(hwnd, screen_pos)
            {
                return Some(window);
            }

            let Some(next) = super::win32::next_window_in_z_order(hwnd) else {
                break;
            };
            hwnd = next;
        }

        // If we only managed to hit the preferred-not window, retry using a full top-level z-order
        // walk. Some window relationships (e.g. owned/topmost windows) can cause a `GW_HWNDNEXT`
        // walk rooted at `WindowFromPoint` to miss windows in a different z-order band.
        if fallback.is_some() && prefer_not_hwnd.is_some() {
            // Prefer enumerating all top-level windows in z-order; this is more reliable than
            // `GetTopWindow + GW_HWNDNEXT` for crossing z-order bands.
            let ordered = super::win32::enum_windows_z_order();
            for hwnd in ordered {
                if hwnd == 0 {
                    continue;
                }

                if prefer_not_hwnd.is_some_and(|p| p == hwnd) {
                    if let Some(&window) = hwnd_to_window.get(&hwnd)
                        && super::win32::screen_pos_in_hwnd(hwnd, screen_pos)
                    {
                        fallback = Some(window);
                    }
                    continue;
                }

                if let Some(&window) = hwnd_to_window.get(&hwnd)
                    && super::win32::screen_pos_in_hwnd(hwnd, screen_pos)
                {
                    return Some(window);
                }
            }
        }

        fallback
    }

    pub(super) fn window_under_cursor_platform(
        &self,
        _screen_pos: PhysicalPosition<f64>,
        _prefer_not: Option<fret_core::AppWindowId>,
    ) -> WindowUnderCursorHit {
        #[cfg(target_os = "macos")]
        if let Some(window) = self.window_under_cursor_macos(_screen_pos, _prefer_not) {
            return WindowUnderCursorHit {
                window: Some(window),
                source: fret_runtime::WindowUnderCursorSource::PlatformMacos,
            };
        }

        #[cfg(target_os = "windows")]
        if let Some(window) = self.window_under_cursor_win32(_screen_pos, _prefer_not) {
            return WindowUnderCursorHit {
                window: Some(window),
                source: fret_runtime::WindowUnderCursorSource::PlatformWin32,
            };
        }

        WindowUnderCursorHit {
            window: None,
            source: fret_runtime::WindowUnderCursorSource::Unknown,
        }
    }

    pub(super) fn window_under_cursor_best_effort(
        &self,
        screen_pos: PhysicalPosition<f64>,
        prefer_not: Option<fret_core::AppWindowId>,
    ) -> WindowUnderCursorHit {
        let platform = self.window_under_cursor_platform(screen_pos, prefer_not);
        if platform.window.is_some() {
            return platform;
        }

        let mut fallback: Option<fret_core::AppWindowId> = None;
        let mut fallback_source = fret_runtime::WindowUnderCursorSource::Unknown;
        for &w in self.windows_z_order.iter().rev() {
            let Some(state) = self.windows.get(w) else {
                continue;
            };
            let Ok(outer) = state.window.outer_position() else {
                continue;
            };
            let deco = state.window.surface_position();
            let size = state.window.surface_size();
            let left = outer.x as f64 + deco.x as f64;
            let top = outer.y as f64 + deco.y as f64;
            let right = left + size.width as f64;
            let bottom = top + size.height as f64;
            if screen_pos.x >= left
                && screen_pos.x < right
                && screen_pos.y >= top
                && screen_pos.y < bottom
            {
                if prefer_not.is_some_and(|p| p == w) {
                    fallback = Some(w);
                    fallback_source = fret_runtime::WindowUnderCursorSource::HeuristicZOrder;
                    continue;
                }
                return WindowUnderCursorHit {
                    window: Some(w),
                    source: fret_runtime::WindowUnderCursorSource::HeuristicZOrder,
                };
            }
        }
        // Fallback if the z-order list has drifted.
        for w in self.windows.keys() {
            if self.windows_z_order.contains(&w) {
                continue;
            }
            let Some(state) = self.windows.get(w) else {
                continue;
            };
            let Ok(outer) = state.window.outer_position() else {
                continue;
            };
            let deco = state.window.surface_position();
            let size = state.window.surface_size();
            let left = outer.x as f64 + deco.x as f64;
            let top = outer.y as f64 + deco.y as f64;
            let right = left + size.width as f64;
            let bottom = top + size.height as f64;
            if screen_pos.x >= left
                && screen_pos.x < right
                && screen_pos.y >= top
                && screen_pos.y < bottom
            {
                if prefer_not.is_some_and(|p| p == w) {
                    fallback = Some(w);
                    fallback_source = fret_runtime::WindowUnderCursorSource::HeuristicRects;
                    continue;
                }
                return WindowUnderCursorHit {
                    window: Some(w),
                    source: fret_runtime::WindowUnderCursorSource::HeuristicRects,
                };
            }
        }
        WindowUnderCursorHit {
            window: fallback,
            source: if fallback.is_some() {
                fallback_source
            } else {
                fret_runtime::WindowUnderCursorSource::Unknown
            },
        }
    }

    pub(super) fn bump_window_z_order(&mut self, window: fret_core::AppWindowId) {
        if self.windows.get(window).is_none() {
            return;
        }
        self.windows_z_order.retain(|w| *w != window);
        self.windows_z_order.push(window);

        #[cfg(target_os = "macos")]
        {
            self.enqueue_window_front(window, None, None, Instant::now());
        }
    }
}
