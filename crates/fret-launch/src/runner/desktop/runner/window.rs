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
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                    let ns_view: id = h.ns_view.as_ptr() as id;
                    if ns_view.is_null() {
                        std::ptr::null_mut()
                    } else {
                        msg_send![ns_view, window]
                    }
                }
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
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                    let ns_view: id = h.ns_view.as_ptr() as id;
                    if ns_view.is_null() {
                        std::ptr::null_mut()
                    } else {
                        msg_send![ns_view, window]
                    }
                }
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
