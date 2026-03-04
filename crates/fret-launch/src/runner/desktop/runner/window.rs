use super::*;
use std::{sync::Arc, time::Duration};

use fret_core::time::Instant;
use fret_core::{Point, Scene};
use fret_render::SurfaceState;
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::collections::HashMap;
use winit::{dpi::PhysicalPosition, window::Window};

#[cfg(target_os = "windows")]
use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

pub(super) struct WindowRuntime<S> {
    pub(super) window: Arc<dyn Window>,
    pub(super) accessibility: Option<fret_runner_winit::accessibility::WinitAccessibility>,
    pub(super) last_accessibility_snapshot: Option<std::sync::Arc<fret_core::SemanticsSnapshot>>,
    pub(super) surface: Option<SurfaceState<'static>>,
    pub(super) scene: Scene,
    pub(super) platform: fret_runner_winit::WinitPlatform,
    /// Coalesced wheel delta awaiting delivery at the next frame boundary.
    ///
    /// When enabled via `FRET_WINIT_COALESCE_WHEEL=1`, we buffer wheel deltas and deliver at most
    /// one `PointerEvent::Wheel` per frame, applying a per-axis max-abs cap and carrying any
    /// remainder over to subsequent frames.
    pub(super) pending_wheel: Option<PendingWheelEvent>,
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

#[derive(Debug, Clone, Copy)]
pub(super) struct PendingWheelEvent {
    pub(super) pointer_id: fret_core::PointerId,
    pub(super) position: Point,
    pub(super) delta: Point,
    pub(super) modifiers: fret_core::Modifiers,
    pub(super) pointer_type: fret_core::PointerType,
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
    pub(super) transparent_payload_applied: bool,
    pub(super) mouse_passthrough_applied: bool,
    pub(super) always_on_top_applied: bool,
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

#[derive(Debug, Clone, Copy)]
pub(super) struct WindowUnderCursorHit {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) source: fret_runtime::WindowUnderCursorSource,
}

#[cfg(target_os = "macos")]
pub(super) fn bring_window_to_front(window: &dyn Window, sender: Option<&dyn Window>) -> bool {
    use objc::runtime::Class;
    use objc::runtime::Object;
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
        let nil: *mut Object = std::ptr::null_mut();
        let Some(class) = Class::get("NSApplication") else {
            window.focus_window();
            return true;
        };
        let app: *mut Object = msg_send![class, sharedApplication];
        if app.is_null() {
            window.focus_window();
            return true;
        }

        let ns_window: *mut Object = match window.window_handle() {
            Ok(handle) => match handle.as_raw() {
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                    let ns_view: *mut Object = h.ns_view.as_ptr() as *mut Object;
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

        let sender_ns_window: *mut Object = match sender_window.window_handle() {
            Ok(handle) => match handle.as_raw() {
                winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                    let ns_view: *mut Object = h.ns_view.as_ptr() as *mut Object;
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

        let key_window_after: *mut Object = msg_send![app, keyWindow];
        let main_window_after: *mut Object = msg_send![app, mainWindow];
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

#[cfg(target_os = "windows")]
pub(super) fn bring_window_to_front(window: &dyn Window, _sender: Option<&dyn Window>) -> bool {
    let hwnd = window
        .window_handle()
        .ok()
        .and_then(|h| match h.as_raw() {
            RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
            _ => None,
        })
        .unwrap_or(0);

    if super::win32::raise_hwnd_to_front(hwnd) {
        return true;
    }

    window.focus_window();
    true
}

#[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
pub(super) fn bring_window_to_front(window: &dyn Window, _sender: Option<&dyn Window>) -> bool {
    window.focus_window();
    true
}

#[cfg(target_os = "macos")]
pub(super) fn set_window_opacity(window: &dyn Window, opacity: f32) -> bool {
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
        return false;
    }

    unsafe {
        let alpha = (opacity.clamp(0.0, 1.0)) as f64;
        let _: () = msg_send![ns_window, setAlphaValue: alpha];
    }
    true
}

#[cfg(target_os = "macos")]
pub(super) fn set_window_mouse_passthrough(window: &dyn Window, enabled: bool) -> bool {
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
        return false;
    }

    unsafe {
        let ignore: bool = enabled;
        let _: () = msg_send![ns_window, setIgnoresMouseEvents: ignore];
    }
    true
}

#[cfg(target_os = "windows")]
pub(super) fn set_window_opacity(window: &dyn Window, opacity: f32) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get() as isize,
            _ => 0,
        },
        Err(_) => 0,
    };
    if hwnd == 0 {
        return false;
    }
    super::win32::set_window_alpha(hwnd, opacity);
    true
}

#[cfg(target_os = "windows")]
pub(super) fn set_window_mouse_passthrough(window: &dyn Window, enabled: bool) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get() as isize,
            _ => 0,
        },
        Err(_) => 0,
    };
    if hwnd == 0 {
        return false;
    }
    super::win32::set_window_mouse_passthrough(hwnd, enabled);
    true
}

#[cfg(target_os = "windows")]
pub(super) fn set_window_background_material(
    window: &dyn Window,
    material: fret_runtime::WindowBackgroundMaterialRequest,
) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get() as isize,
            _ => 0,
        },
        Err(_) => 0,
    };
    if hwnd == 0 {
        return false;
    }

    let ty = match material {
        fret_runtime::WindowBackgroundMaterialRequest::None => {
            super::win32::dwm_system_backdrop_type_for_none()
        }
        fret_runtime::WindowBackgroundMaterialRequest::SystemDefault => {
            super::win32::dwm_system_backdrop_type_for_system_default()
        }
        fret_runtime::WindowBackgroundMaterialRequest::Mica => {
            super::win32::dwm_system_backdrop_type_for_mica()
        }
        fret_runtime::WindowBackgroundMaterialRequest::Acrylic => {
            super::win32::dwm_system_backdrop_type_for_acrylic()
        }
        fret_runtime::WindowBackgroundMaterialRequest::Vibrancy => {
            // macOS-only; should have been clamped by capabilities.
            return false;
        }
    };

    super::win32::set_dwm_system_backdrop_type(hwnd, ty)
}

#[cfg(target_os = "macos")]
pub(super) fn set_window_background_material(
    window: &dyn Window,
    material: fret_runtime::WindowBackgroundMaterialRequest,
) -> bool {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use winit::raw_window_handle::HasWindowHandle as _;

    // We implement "Vibrancy" using an `NSVisualEffectView` behind winit's view.
    //
    // This is intentionally best-effort:
    // - we do not use private APIs for older macOS versions,
    // - we avoid hardcoding `NSVisualEffectMaterial` values and rely on defaults.
    const IDENT: &str = "fret.vibrancy.background.v1";

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct NsPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct NsSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Debug, Default)]
    struct NsRect {
        origin: NsPoint,
        size: NsSize,
    }

    fn ns_string(s: &str) -> *mut Object {
        let Some(cls) = Class::get("NSString") else {
            return std::ptr::null_mut();
        };
        let Ok(cstr) = std::ffi::CString::new(s) else {
            return std::ptr::null_mut();
        };
        // SAFETY: `stringWithUTF8String:` copies the bytes immediately.
        unsafe { msg_send![cls, stringWithUTF8String: cstr.as_ptr()] }
    }

    let (ns_view, ns_window): (*mut Object, *mut Object) = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::AppKit(h) => {
                let ns_view: *mut Object = h.ns_view.as_ptr() as *mut Object;
                if ns_view.is_null() {
                    (std::ptr::null_mut(), std::ptr::null_mut())
                } else {
                    let ns_window: *mut Object = unsafe { msg_send![ns_view, window] };
                    (ns_view, ns_window)
                }
            }
            _ => (std::ptr::null_mut(), std::ptr::null_mut()),
        },
        Err(_) => (std::ptr::null_mut(), std::ptr::null_mut()),
    };
    if ns_view.is_null() || ns_window.is_null() {
        return false;
    }

    // SAFETY: these Objective-C calls are best-effort and return `nil` on failure.
    unsafe {
        let content_view: *mut Object = msg_send![ns_window, contentView];
        if content_view.is_null() {
            return false;
        }
        // IMPORTANT: winit's AppKit handle is the *view* that hosts the GPU surface. If we add an
        // `NSVisualEffectView` as a subview *inside* that view, it will sit above the surface and
        // can cover the UI (manifesting as a solid/blurred "white block").
        //
        // Therefore we attach the effect view as a sibling below winit's view. We intentionally
        // do NOT replace the window `contentView` because winit-appkit expects to own it.
        let mut container_view: *mut Object = msg_send![ns_view, superview];
        if container_view.is_null() {
            container_view = content_view;
        }
        if container_view.is_null() {
            return false;
        }

        super::macos_window_log(format_args!(
            "[bg-material] winit={:?} material={:?} ns_view={:p} content_view={:p} container_view={:p}",
            window.id(),
            material,
            ns_view as *const std::ffi::c_void,
            content_view as *const std::ffi::c_void,
            container_view as *const std::ffi::c_void,
        ));
        let subviews: *mut Object = msg_send![container_view, subviews];
        let count: usize = if subviews.is_null() {
            0
        } else {
            msg_send![subviews, count]
        };

        let wanted_ident = ns_string(IDENT);
        let mut existing: *mut Object = std::ptr::null_mut();
        if !wanted_ident.is_null() {
            for i in 0..count {
                let v: *mut Object = msg_send![subviews, objectAtIndex: i];
                if v.is_null() {
                    continue;
                }
                let has_identifier: bool = msg_send![v, respondsToSelector: sel!(identifier)];
                if !has_identifier {
                    continue;
                }
                let ident: *mut Object = msg_send![v, identifier];
                if !ident.is_null() {
                    let is_eq: bool = msg_send![ident, isEqualToString: wanted_ident];
                    if is_eq {
                        existing = v;
                        break;
                    }
                }
            }
        }

        let enable = matches!(
            material,
            fret_runtime::WindowBackgroundMaterialRequest::Vibrancy
                | fret_runtime::WindowBackgroundMaterialRequest::SystemDefault
        );
        if !enable {
            if !existing.is_null() {
                let _: () = msg_send![existing, removeFromSuperview];
            }
            return matches!(
                material,
                fret_runtime::WindowBackgroundMaterialRequest::None
            );
        }

        // Ensure the window is non-opaque so the compositor can blend the surface alpha and the
        // behind-window material can show through.
        let _: () = msg_send![ns_window, setOpaque: false];
        if let Some(color_cls) = Class::get("NSColor") {
            // Avoid `clearColor` to preserve window shadow.
            let bg: *mut Object = msg_send![
                color_cls,
                colorWithSRGBRed: 0f64
                green: 0f64
                blue: 0f64
                alpha: 0.0001f64
            ];
            if !bg.is_null() {
                let _: () = msg_send![ns_window, setBackgroundColor: bg];
            }
        }

        let container_bounds: NsRect = msg_send![container_view, bounds];
        let effect_material: u64 = match material {
            // NSVisualEffectMaterialUnderWindowBackground (17) produces an explicit blurred
            // "material" under the window content.
            fret_runtime::WindowBackgroundMaterialRequest::Vibrancy => 17,
            // NSVisualEffectMaterialWindowBackground (12) is closer to the default background
            // appearance (best-effort).
            fret_runtime::WindowBackgroundMaterialRequest::SystemDefault => 12,
            _ => 12,
        };

        if existing.is_null() {
            let Some(cls) = Class::get("NSVisualEffectView") else {
                return false;
            };
            let frame: NsRect = container_bounds;
            let view: *mut Object = msg_send![cls, alloc];
            let view: *mut Object = msg_send![view, initWithFrame: frame];
            if view.is_null() {
                return false;
            }

            // `NSVisualEffectView` should resize with the content view.
            //
            // NSViewWidthSizable (2) | NSViewHeightSizable (16)
            let _: () = msg_send![view, setAutoresizingMask: 18u64];
            if !wanted_ident.is_null() {
                let _: () = msg_send![view, setIdentifier: wanted_ident];
            }

            // Prefer a behind-window effect so we get true desktop/backdrop blur.
            // NSVisualEffectBlendingModeBehindWindow (0)
            let _: () = msg_send![view, setBlendingMode: 0u64];
            // NSVisualEffectStateActive (1)
            let _: () = msg_send![view, setState: 1u64];
            let _: () = msg_send![view, setMaterial: effect_material];

            // Insert below winit's view so input continues to flow to the UI.
            //
            // NSWindowOrderingModeBelow (-1)
            let _: () = msg_send![
                container_view,
                addSubview: view
                positioned: -1i64
                relativeTo: ns_view
            ];
            super::macos_window_log(format_args!(
                "[bg-material-attach] winit={:?} effect_view={:p} action=create",
                window.id(),
                view as *const std::ffi::c_void,
            ));
        } else {
            // If the view already exists, keep it sized and ensure it stays *behind* winit's view.
            //
            // We reinsert it because some view-tree mutations (or initial attachment as a subview
            // of `ns_view`) can accidentally place it above the GPU surface.
            let _: () = msg_send![existing, setFrame: container_bounds];
            let _: () = msg_send![existing, setAutoresizingMask: 18u64];
            let _: () = msg_send![existing, removeFromSuperview];
            let _: () = msg_send![
                container_view,
                addSubview: existing
                positioned: -1i64
                relativeTo: ns_view
            ];
            let _: () = msg_send![existing, setBlendingMode: 0u64];
            let _: () = msg_send![existing, setState: 1u64];
            let _: () = msg_send![existing, setMaterial: effect_material];
            super::macos_window_log(format_args!(
                "[bg-material-attach] winit={:?} effect_view={:p} action=reinsert",
                window.id(),
                existing as *const std::ffi::c_void,
            ));
        }

        true
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub(super) fn set_window_opacity(_window: &dyn Window, _opacity: f32) -> bool {
    false
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub(super) fn set_window_mouse_passthrough(_window: &dyn Window, _enabled: bool) -> bool {
    false
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub(super) fn set_window_background_material(
    _window: &dyn Window,
    _material: fret_runtime::WindowBackgroundMaterialRequest,
) -> bool {
    false
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
    const WINDOW_VISIBILITY_PADDING_PX: f64 = 40.0;

    #[cfg(target_os = "macos")]
    fn ns_window_number_for_window(window: &dyn Window) -> Option<i32> {
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
    fn ordered_ns_window_numbers_front_to_back() -> Vec<i32> {
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

    pub(super) fn compute_window_position_from_anchor(
        &self,
        anchor: fret_core::WindowAnchor,
    ) -> Option<WindowPosition> {
        let anchor_state = self.windows.get(anchor.window)?;
        // `WindowAnchor::position` is in surface-local logical coordinates (matching pointer
        // events), so start from the surface origin in desktop coordinates.
        let outer = anchor_state.window.outer_position().ok()?;
        let surface = anchor_state.window.surface_position();
        let scale = anchor_state.window.scale_factor();

        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = outer.x as f64 + surface.x as f64 + anchor.position.x.0 as f64 * scale + ox;
        let mut y = outer.y as f64 + surface.y as f64 + anchor.position.y.0 as f64 * scale + oy;

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        if let Some(monitor) = anchor_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            // Leave a small margin so the window stays reachable even if its size is larger
            // than the monitor work area.
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_position_from_cursor(
        &self,
        reference_window: fret_core::AppWindowId,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let ref_state = self.windows.get(reference_window)?;
        let (ox, oy) = self.config.new_window_anchor_offset;
        let mut x = screen_pos.x + ox;
        let mut y = screen_pos.y + oy;

        if let Some(monitor) = ref_state.window.current_monitor()
            && let (Some(pos), Some(mode)) = (monitor.position(), monitor.current_video_mode())
        {
            let size = mode.size();
            let min_x = pos.x as f64;
            let min_y = pos.y as f64;
            let max_x = min_x + size.width as f64 - 40.0;
            let max_y = min_y + size.height as f64 - 40.0;

            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_position_from_cursor_grab_estimate(
        &self,
        reference_window: fret_core::AppWindowId,
        new_window_inner_size: WindowLogicalSize,
        grab_offset_logical: Point,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(reference_window)?;
        let scale = state.window.scale_factor();

        let max_client = winit::dpi::LogicalSize::new(
            new_window_inner_size.width as f32,
            new_window_inner_size.height as f32,
        );

        let mut x = screen_pos.x;
        let mut y = screen_pos.y;

        #[cfg(target_os = "windows")]
        let decoration_offset = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::decoration_offset_for_hwnd)
            .unwrap_or_else(|| state.window.surface_position());
        #[cfg(not(target_os = "windows"))]
        let decoration_offset = state.window.surface_position();

        if let Some((ox, oy)) = outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            scale,
            decoration_offset,
            Some(max_client),
        ) {
            x = ox;
            y = oy;
        }

        // Best-effort clamping: avoid creating "off-screen" floating windows due to
        // platform-specific coordinate spaces and DPI conversions.
        let outer_size =
            winit::dpi::LogicalSize::new(new_window_inner_size.width, new_window_inner_size.height)
                .to_physical::<u32>(scale);

        #[cfg(target_os = "windows")]
        if let Some(work) = super::win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn compute_window_outer_position_from_cursor_grab(
        &self,
        target_window: fret_core::AppWindowId,
        grab_offset_logical: Point,
    ) -> Option<WindowPosition> {
        let screen_pos = self.cursor_screen_pos?;
        let state = self.windows.get(target_window)?;
        let scale = state.window.scale_factor();

        // Clamp the grab point to the target window's current client size. During tear-off, the
        // grab offset comes from the source window's client coordinates; if the new floating
        // window is smaller, keeping the original offset would place the cursor outside the new
        // window (visible as a fixed offset between cursor and window).
        let target_inner = state.window.surface_size();
        let target_inner_logical: winit::dpi::LogicalSize<f32> = target_inner.to_logical(scale);

        #[cfg(target_os = "windows")]
        let decoration_offset = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::decoration_offset_for_hwnd)
            .unwrap_or_else(|| state.window.surface_position());
        #[cfg(not(target_os = "windows"))]
        let decoration_offset = state.window.surface_position();

        let (mut x, mut y) = outer_pos_for_cursor_grab(
            screen_pos,
            grab_offset_logical,
            scale,
            decoration_offset,
            Some(target_inner_logical),
        )?;

        // Align with ImGui docking/multi-viewport behavior:
        // - platform backend sets the window pos as requested
        // - visibility/reachability constraints are based on the *target monitor*, not the window's
        //   current monitor (which can pin the window at monitor edges).
        let outer_size = state.window.outer_size();

        #[cfg(target_os = "windows")]
        if let Some(work) = super::win32::monitor_work_area_for_point(screen_pos) {
            (x, y) = Self::clamp_window_outer_pos_to_monitor(
                x,
                y,
                outer_size,
                work,
                Self::WINDOW_VISIBILITY_PADDING_PX,
            );
        } else {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            let monitors = Self::monitor_rects_physical(state.window.as_ref());
            if let Some(idx) = Self::find_monitor_for_point(&monitors, screen_pos)
                && let Some(monitor) = monitors.get(idx).copied()
            {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            } else if let Some(monitor) = Self::virtual_desktop_bounds(state.window.as_ref()) {
                (x, y) = Self::clamp_window_outer_pos_to_monitor(
                    x,
                    y,
                    outer_size,
                    monitor,
                    Self::WINDOW_VISIBILITY_PADDING_PX,
                );
            }
        }

        Some(WindowPosition::Physical(WindowPhysicalPosition::new(
            x.round() as i32,
            y.round() as i32,
        )))
    }

    pub(super) fn cursor_screen_pos_fallback_for_window(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<PhysicalPosition<f64>> {
        let state = self.windows.get(window)?;
        // `Window::surface_position()` is defined as the decoration offset from the outer
        // window position to the client/surface origin (ImGui-style multi-viewport contract).
        // Convert it to a screen-space client origin before adding a local cursor position.
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            })?;
        #[cfg(not(target_os = "windows"))]
        let origin = {
            let outer = state.window.outer_position().ok()?;
            let deco = state.window.surface_position();
            client_origin_screen(outer, deco)
        };
        let scale = state.window.scale_factor();
        let x = origin.x + state.platform.input.cursor_pos.x.0 as f64 * scale;
        let y = origin.y + state.platform.input.cursor_pos.y.0 as f64 * scale;
        Some(PhysicalPosition::new(x, y))
    }

    pub(super) fn screen_pos_in_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> bool {
        let Some(state) = self.windows.get(window) else {
            return false;
        };
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            });
        #[cfg(not(target_os = "windows"))]
        let origin = state
            .window
            .outer_position()
            .ok()
            .map(|outer| client_origin_screen(outer, state.window.surface_position()));
        let size = state.window.surface_size();
        origin.is_some_and(|origin| screen_pos_in_client(origin, size, screen_pos))
    }

    pub(super) fn local_pos_for_window(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<Point> {
        let state = self.windows.get(window)?;
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            })?;
        #[cfg(not(target_os = "windows"))]
        let origin = {
            let outer = state.window.outer_position().ok()?;
            let deco = state.window.surface_position();
            client_origin_screen(outer, deco)
        };
        Some(local_pos_for_screen_pos(
            origin,
            state.window.scale_factor(),
            screen_pos,
        ))
    }

    pub(super) fn window_client_rect_screen(
        &self,
        window: fret_core::AppWindowId,
    ) -> Option<(
        winit::dpi::PhysicalPosition<f64>,
        winit::dpi::PhysicalSize<u32>,
    )> {
        let state = self.windows.get(window)?;
        #[cfg(target_os = "windows")]
        let origin = Self::hwnd_for_window(state.window.as_ref())
            .and_then(super::win32::client_origin_screen_for_hwnd)
            .or_else(|| {
                let outer = state.window.outer_position().ok()?;
                let deco = Self::hwnd_for_window(state.window.as_ref())
                    .and_then(super::win32::decoration_offset_for_hwnd)
                    .unwrap_or_else(|| state.window.surface_position());
                Some(client_origin_screen(outer, deco))
            })?;
        #[cfg(not(target_os = "windows"))]
        let origin = {
            let outer = state.window.outer_position().ok()?;
            let deco = state.window.surface_position();
            client_origin_screen(outer, deco)
        };
        let size = state.window.surface_size();
        Some((origin, size))
    }

    pub(super) fn clamp_screen_pos_to_window_client(
        &self,
        window: fret_core::AppWindowId,
        screen_pos: PhysicalPosition<f64>,
    ) -> Option<PhysicalPosition<f64>> {
        let (origin, size) = self.window_client_rect_screen(window)?;
        if size.width == 0 || size.height == 0 {
            return None;
        }
        // Clamp to the inclusive interior to avoid points right on the boundary (which can be
        // sensitive to rounding and platform hit-test behavior).
        let min_x = origin.x + 1.0;
        let min_y = origin.y + 1.0;
        let max_x = origin.x + (size.width as f64) - 1.0;
        let max_y = origin.y + (size.height as f64) - 1.0;
        Some(PhysicalPosition::new(
            screen_pos.x.clamp(min_x, max_x),
            screen_pos.y.clamp(min_y, max_y),
        ))
    }

    pub(super) fn window_under_cursor_platform(
        &self,
        screen_pos: PhysicalPosition<f64>,
        prefer_not: Option<fret_core::AppWindowId>,
    ) -> WindowUnderCursorHit {
        #[cfg(target_os = "macos")]
        if let Some(window) = self.window_under_cursor_macos(screen_pos, prefer_not) {
            return WindowUnderCursorHit {
                window: Some(window),
                source: fret_runtime::WindowUnderCursorSource::PlatformMacos,
            };
        }

        #[cfg(target_os = "windows")]
        if let Some(window) = self.window_under_cursor_win32(screen_pos, prefer_not) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::{PhysicalPosition, PhysicalSize};

    #[test]
    fn outer_pos_for_cursor_grab_accounts_for_decorations_and_scale() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(20.0), Px(40.0));
        let scale = 1.5;
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let max_client = winit::dpi::LogicalSize::new(200.0f32, 200.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 960.0);
        assert_eq!(y, 410.0);
    }

    #[test]
    fn outer_pos_for_cursor_grab_clamps_to_client_size() {
        let cursor = PhysicalPosition::new(1000.0, 500.0);
        let grab = Point::new(Px(9999.0), Px(9999.0));
        let scale = 2.0;
        let deco = winit::dpi::PhysicalPosition::new(0, 0);
        let max_client = winit::dpi::LogicalSize::new(100.0f32, 100.0f32);

        let (x, y) = outer_pos_for_cursor_grab(cursor, grab, scale, deco, Some(max_client))
            .expect("expected outer pos");
        assert_eq!(x, 800.0);
        assert_eq!(y, 300.0);
    }

    #[test]
    fn client_origin_screen_adds_decoration_offset() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(12, 34);
        let origin = client_origin_screen(outer, deco);
        assert_eq!(origin, PhysicalPosition::new(112.0, 234.0));
    }

    #[test]
    fn screen_pos_in_client_uses_half_open_bounds() {
        let origin = PhysicalPosition::new(10.0, 20.0);
        let size = PhysicalSize::new(100u32, 50u32);

        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 20.0)
        ));
        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(109.9, 69.9)
        ));

        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(110.0, 20.0)
        ));
        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(10.0, 70.0)
        ));
    }

    #[test]
    fn local_pos_for_screen_pos_respects_scale_factor() {
        let origin = PhysicalPosition::new(100.0, 200.0);
        let scale = 2.0;
        let screen_pos = PhysicalPosition::new(120.0, 240.0);
        let local = local_pos_for_screen_pos(origin, scale, screen_pos);
        assert_eq!(local, Point::new(Px(10.0), Px(20.0)));
    }

    #[test]
    fn screen_pos_in_client_respects_outer_plus_decoration_offset() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(12, 34);
        let origin = client_origin_screen(outer, deco);
        let size = PhysicalSize::new(100u32, 50u32);

        assert!(screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(112.0, 234.0)
        ));
        assert!(!screen_pos_in_client(
            origin,
            size,
            PhysicalPosition::new(111.9, 234.0)
        ));
    }

    #[test]
    fn local_pos_for_screen_pos_roundtrips_with_outer_plus_decoration_and_scale() {
        let outer = winit::dpi::PhysicalPosition::new(100, 200);
        let deco = winit::dpi::PhysicalPosition::new(10, 30);
        let origin = client_origin_screen(outer, deco);
        let scale = 1.5;

        let desired_local = Point::new(Px(20.0), Px(40.0));
        let screen_pos = PhysicalPosition::new(
            origin.x + desired_local.x.0 as f64 * scale,
            origin.y + desired_local.y.0 as f64 * scale,
        );

        let local = local_pos_for_screen_pos(origin, scale, screen_pos);
        assert_eq!(local, desired_local);
    }
}
