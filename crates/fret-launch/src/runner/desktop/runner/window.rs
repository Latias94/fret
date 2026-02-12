use super::*;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use fret_core::{Point, Scene};
use fret_render::SurfaceState;
use winit::{dpi::PhysicalPosition, window::Window};

pub(super) struct WindowRuntime<S> {
    pub(super) window: Arc<dyn Window>,
    pub(super) accessibility: Option<fret_runner_winit::accessibility::WinitAccessibility>,
    pub(super) last_accessibility_snapshot: Option<std::sync::Arc<fret_core::SemanticsSnapshot>>,
    pub(super) surface: Option<SurfaceState<'static>>,
    pub(super) scene: Scene,
    pub(super) platform: fret_runner_winit::WinitPlatform,
    #[cfg(target_os = "android")]
    pub(super) android_bottom_inset_baseline: Option<fret_core::Px>,
    /// Coalesced resizes awaiting application at the next frame boundary.
    ///
    /// During interactive window resize, platforms may emit multiple size updates per vblank.
    /// We keep only the latest physical size and apply it once per `RedrawRequested` to avoid
    /// reconfiguring the surface and recomputing layout more often than we can present.
    pub(super) pending_surface_resize: Option<winit::dpi::PhysicalSize<u32>>,
    /// Last delivered (quantized) logical size for `Event::WindowResized`.
    ///
    /// This mirrors GPUI's `set_frame_size` guard (`old_size == new_size`) and helps reduce
    /// float-noise churn in window-metrics consumers during interactive resize.
    pub(super) last_delivered_window_resized: Option<(u32, u32)>,
    pub(super) is_focused: bool,
    pub(super) external_drag_files: Vec<std::path::PathBuf>,
    pub(super) external_drag_token: Option<fret_runtime::ExternalDropToken>,
    pub(super) user: S,
    #[cfg(windows)]
    pub(super) os_menu: Option<super::windows_menu::WindowsMenuBar>,
}

#[derive(Debug, Clone)]
pub(super) struct PendingFrontRequest {
    pub(super) source_window: Option<fret_core::AppWindowId>,
    pub(super) panel: Option<fret_core::PanelKey>,
    pub(super) created_at: Instant,
    pub(super) next_attempt_at: Instant,
    pub(super) attempts_left: u8,
}

#[derive(Debug, Clone)]
pub(super) struct TimerEntry {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) deadline: Instant,
    pub(super) repeat: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DockTearoffFollow {
    pub(super) window: fret_core::AppWindowId,
    pub(super) source_window: fret_core::AppWindowId,
    pub(super) grab_offset: Point,
    pub(super) manual_follow: bool,
    pub(super) last_outer_pos: Option<PhysicalPosition<i32>>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MonitorRectF64 {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RectF64 {
    pub(super) min_x: f64,
    pub(super) min_y: f64,
    pub(super) max_x: f64,
    pub(super) max_y: f64,
}

#[cfg(target_os = "macos")]
pub(super) fn bring_window_to_front(window: &dyn Window, sender: Option<&dyn Window>) -> bool {
    use cocoa::base::{id, nil};
    use objc::runtime::Class;
    use objc::{msg_send, sel, sel_impl};
    use winit::raw_window_handle::HasWindowHandle as _;

    let Some(sender_window) = sender else {
        window.focus_window();
        return true;
    };
    if sender_window.id() == window.id() {
        window.focus_window();
        return true;
    }

    unsafe {
        let Some(class) = Class::get("NSApplication") else {
            window.focus_window();
            return true;
        };
        let app: id = msg_send![class, sharedApplication];
        if app.is_null() {
            window.focus_window();
            return true;
        }

        let ns_window: id = match window.window_handle() {
            Ok(handle) => match handle.as_raw() {
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => h.ns_window.as_ptr() as id,
                _ => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        };
        if ns_window.is_null() {
            window.focus_window();
            return true;
        }

        let sender_ns_window: id = match sender_window.window_handle() {
            Ok(handle) => match handle.as_raw() {
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => h.ns_window.as_ptr() as id,
                _ => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        };
        if sender_ns_window.is_null() {
            window.focus_window();
            return true;
        }

        let sender_level: i64 = msg_send![sender_ns_window, level];
        let sender_number: i32 = msg_send![sender_ns_window, windowNumber];
        let sender_ordered_index: i32 = msg_send![sender_ns_window, orderedIndex];
        let sender_occlusion: u64 = msg_send![sender_ns_window, occlusionState];
        super::macos_window_log(format_args!(
            "[raise-before] target={:p} sender={:p} sender_level={} sender_num={} sender_ordered_index={} sender_occl=0x{:x} winit={:?}",
            ns_window as *const std::ffi::c_void,
            sender_ns_window as *const std::ffi::c_void,
            sender_level,
            sender_number,
            sender_ordered_index,
            sender_occlusion,
            window.id(),
        ));

        let _: () = msg_send![app, activateIgnoringOtherApps: true];

        let _: () = msg_send![ns_window, makeKeyAndOrderFront: nil];
        let _: () = msg_send![ns_window, orderFrontRegardless];

        // Keep winit’s internal focus bookkeeping aligned; in practice this also improves the
        // success rate of the ordering change when the source window is in a tracked interaction.
        window.focus_window();

        let key_window_after: id = msg_send![app, keyWindow];
        let main_window_after: id = msg_send![app, mainWindow];
        let is_key_after: bool = msg_send![ns_window, isKeyWindow];
        let is_main_after: bool = msg_send![ns_window, isMainWindow];
        let is_visible_after: bool = msg_send![ns_window, isVisible];
        let occlusion_after: u64 = msg_send![ns_window, occlusionState];
        let level_after: i64 = msg_send![ns_window, level];
        let ordered_index_after: i32 = msg_send![ns_window, orderedIndex];
        let window_number_after: i32 = msg_send![ns_window, windowNumber];
        super::macos_window_log(format_args!(
            "[raise-after]  target={:p} sender={:p} sender_level={} sender_num={} sender_ordered_index={} sender_occl=0x{:x} key={:p} main={:p} is_key={} is_main={} visible={} occl=0x{:x} level={} ordered_index={} win_num={} winit={:?}",
            ns_window as *const std::ffi::c_void,
            sender_ns_window as *const std::ffi::c_void,
            sender_level,
            sender_number,
            sender_ordered_index,
            sender_occlusion,
            key_window_after as *const std::ffi::c_void,
            main_window_after as *const std::ffi::c_void,
            is_key_after,
            is_main_after,
            is_visible_after,
            occlusion_after,
            level_after,
            ordered_index_after,
            window_number_after,
            window.id(),
        ));
        true
    }
}

#[cfg(not(target_os = "macos"))]
pub(super) fn bring_window_to_front(window: &dyn Window, _sender: Option<&dyn Window>) -> bool {
    window.focus_window();
    true
}

pub(super) fn client_origin_screen(
    outer: winit::dpi::PhysicalPosition<i32>,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
) -> winit::dpi::PhysicalPosition<f64> {
    winit::dpi::PhysicalPosition::new(
        outer.x as f64 + decoration_offset.x as f64,
        outer.y as f64 + decoration_offset.y as f64,
    )
}

pub(super) fn screen_pos_in_client(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    client_size: winit::dpi::PhysicalSize<u32>,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> bool {
    let left = client_origin.x;
    let top = client_origin.y;
    let right = left + client_size.width as f64;
    let bottom = top + client_size.height as f64;
    screen_pos.x >= left && screen_pos.x < right && screen_pos.y >= top && screen_pos.y < bottom
}

pub(super) fn local_pos_for_screen_pos(
    client_origin: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
    screen_pos: winit::dpi::PhysicalPosition<f64>,
) -> Point {
    let local_physical = winit::dpi::PhysicalPosition::new(
        screen_pos.x - client_origin.x,
        screen_pos.y - client_origin.y,
    );
    let local_logical: winit::dpi::LogicalPosition<f32> = local_physical.to_logical(scale_factor);
    Point::new(Px(local_logical.x), Px(local_logical.y))
}

pub(super) fn outer_pos_for_cursor_grab(
    screen_pos: PhysicalPosition<f64>,
    grab_offset_logical: Point,
    scale_factor: f64,
    decoration_offset: winit::dpi::PhysicalPosition<i32>,
    max_client_logical: Option<winit::dpi::LogicalSize<f32>>,
) -> Option<(f64, f64)> {
    if !grab_offset_logical.x.0.is_finite()
        || !grab_offset_logical.y.0.is_finite()
        || grab_offset_logical.x.0 < 0.0
        || grab_offset_logical.y.0 < 0.0
    {
        return None;
    }

    let mut grab_x = grab_offset_logical.x.0;
    let mut grab_y = grab_offset_logical.y.0;
    if let Some(max) = max_client_logical {
        if max.width.is_finite() && max.width > 0.0 {
            grab_x = grab_x.min(max.width).max(0.0);
        } else {
            grab_x = 0.0;
        }
        if max.height.is_finite() && max.height > 0.0 {
            grab_y = grab_y.min(max.height).max(0.0);
        } else {
            grab_y = 0.0;
        }
    }

    // Match ImGui's platform contract:
    // - viewport pos is client/inner screen position (logical)
    // - winit expects outer position
    // - therefore: outer = desired_client - decoration_offset(window)
    // See `repo-ref/dear-imgui-rs/backends/dear-imgui-winit/src/multi_viewport.rs:winit_set_window_pos`.
    let grab_client_x = grab_x as f64 * scale_factor;
    let grab_client_y = grab_y as f64 * scale_factor;
    let grab_outer_x = decoration_offset.x as f64 + grab_client_x;
    let grab_outer_y = decoration_offset.y as f64 + grab_client_y;

    let x = screen_pos.x - grab_outer_x;
    let y = screen_pos.y - grab_outer_y;
    Some((x, y))
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn virtual_desktop_bounds(window: &dyn Window) -> Option<MonitorRectF64> {
        let mut monitors = window.available_monitors();
        let first = monitors.next()?;

        let first_pos = first.position()?;
        let first_size = first.current_video_mode()?.size();
        let mut min_x = first_pos.x as f64;
        let mut min_y = first_pos.y as f64;
        let mut max_x = first_pos.x as f64 + first_size.width as f64;
        let mut max_y = first_pos.y as f64 + first_size.height as f64;

        for monitor in monitors {
            let Some(pos) = monitor.position() else {
                continue;
            };
            let Some(mode) = monitor.current_video_mode() else {
                continue;
            };
            let size = mode.size();
            min_x = min_x.min(pos.x as f64);
            min_y = min_y.min(pos.y as f64);
            max_x = max_x.max(pos.x as f64 + size.width as f64);
            max_y = max_y.max(pos.y as f64 + size.height as f64);
        }

        Some(MonitorRectF64 {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    pub(super) fn monitor_rects_physical(window: &dyn Window) -> Vec<MonitorRectF64> {
        window
            .available_monitors()
            .filter_map(|m| {
                let pos = m.position()?;
                let size = m.current_video_mode()?.size();
                Some(MonitorRectF64 {
                    min_x: pos.x as f64,
                    min_y: pos.y as f64,
                    max_x: pos.x as f64 + size.width as f64,
                    max_y: pos.y as f64 + size.height as f64,
                })
            })
            .collect()
    }

    pub(super) fn find_monitor_for_point(
        monitors: &[MonitorRectF64],
        point: PhysicalPosition<f64>,
    ) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }

        let mut best = 0usize;
        let mut best_dist2 = f64::INFINITY;
        for (i, m) in monitors.iter().enumerate() {
            let dx = if point.x < m.min_x {
                m.min_x - point.x
            } else if point.x > m.max_x {
                point.x - m.max_x
            } else {
                0.0
            };
            let dy = if point.y < m.min_y {
                m.min_y - point.y
            } else if point.y > m.max_y {
                point.y - m.max_y
            } else {
                0.0
            };
            let dist2 = dx * dx + dy * dy;
            if dist2 < best_dist2 {
                best_dist2 = dist2;
                best = i;
            }
            if dist2 == 0.0 {
                return Some(i);
            }
        }

        Some(best)
    }

    pub(super) fn find_monitor_for_rect(
        monitors: &[MonitorRectF64],
        rect: RectF64,
    ) -> Option<usize> {
        if monitors.is_empty() {
            return None;
        }
        if monitors.len() == 1 {
            return Some(0);
        }

        let mut best = 0usize;
        let mut best_area = -1.0f64;
        for (i, m) in monitors.iter().enumerate() {
            let ix0 = rect.min_x.max(m.min_x);
            let iy0 = rect.min_y.max(m.min_y);
            let ix1 = rect.max_x.min(m.max_x);
            let iy1 = rect.max_y.min(m.max_y);
            let iw = (ix1 - ix0).max(0.0);
            let ih = (iy1 - iy0).max(0.0);
            let area = iw * ih;
            if area > best_area {
                best_area = area;
                best = i;
            }
        }
        Some(best)
    }

    pub(super) fn clamp_window_outer_pos_to_monitor(
        desired_outer_x: f64,
        desired_outer_y: f64,
        outer_size: winit::dpi::PhysicalSize<u32>,
        monitor: MonitorRectF64,
        padding: f64,
    ) -> (f64, f64) {
        let w = outer_size.width as f64;
        let h = outer_size.height as f64;

        let pad_x = padding.min(w).max(0.0);
        let pad_y = padding.min(h).max(0.0);

        // Keep at least `pad` pixels of the window visible within the monitor bounds.
        let min_x = monitor.min_x - (w - pad_x);
        let max_x = monitor.max_x - pad_x;
        let min_y = monitor.min_y - (h - pad_y);
        let max_y = monitor.max_y - pad_y;

        let clamped_x = desired_outer_x.clamp(min_x, max_x.max(min_x));
        let clamped_y = desired_outer_y.clamp(min_y, max_y.max(min_y));
        (clamped_x, clamped_y)
    }

    pub(super) fn settle_window_outer_position(
        &self,
        window: &dyn Window,
        cursor_screen_pos: Option<PhysicalPosition<f64>>,
    ) -> Option<PhysicalPosition<i32>> {
        let outer_pos = window.outer_position().ok()?;
        let outer_size = window.outer_size();

        let desired_x = outer_pos.x as f64;
        let desired_y = outer_pos.y as f64;

        #[cfg(target_os = "windows")]
        if let Some(cursor) = cursor_screen_pos
            && let Some(work) = win32::monitor_work_area_for_point(cursor)
        {
            let (x, y) = Self::clamp_window_outer_pos_to_monitor(
                desired_x,
                desired_y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
            let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
            return (target != outer_pos).then_some(target);
        }

        let monitors = Self::monitor_rects_physical(window);
        let monitor = if let Some(cursor) = cursor_screen_pos
            && let Some(idx) = Self::find_monitor_for_point(&monitors, cursor)
            && let Some(m) = monitors.get(idx).copied()
        {
            Some(m)
        } else {
            let rect = RectF64 {
                min_x: desired_x,
                min_y: desired_y,
                max_x: desired_x + outer_size.width as f64,
                max_y: desired_y + outer_size.height as f64,
            };
            let idx = Self::find_monitor_for_rect(&monitors, rect);
            idx.and_then(|i| monitors.get(i).copied())
        };

        let monitor = monitor.or_else(|| Self::virtual_desktop_bounds(window));
        let monitor = monitor?;

        let (x, y) = Self::clamp_window_outer_pos_to_monitor(
            desired_x,
            desired_y,
            outer_size,
            monitor,
            Self::WINDOW_VISIBILITY_PADDING_PX,
        );

        let target = PhysicalPosition::new(x.round() as i32, y.round() as i32);
        if target == outer_pos {
            None
        } else {
            Some(target)
        }
    }
}
