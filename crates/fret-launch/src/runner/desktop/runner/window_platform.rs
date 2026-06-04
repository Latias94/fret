use winit::window::Window;

#[cfg(target_os = "windows")]
use winit::raw_window_handle::{HasWindowHandle as _, RawWindowHandle};

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

        // Keep winit's internal focus bookkeeping aligned; in practice this also improves the
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
pub(super) fn set_window_hit_test_passthrough_all(window: &dyn Window, enabled: bool) -> bool {
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
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get(),
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
pub(super) fn set_window_hit_test_passthrough_all(window: &dyn Window, enabled: bool) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get(),
            _ => 0,
        },
        Err(_) => 0,
    };
    if hwnd == 0 {
        return false;
    }
    super::win32::set_window_hit_test_passthrough_all(hwnd, enabled);
    true
}

#[cfg(target_os = "windows")]
fn set_window_hit_test_passthrough_regions(
    window: &dyn Window,
    regions: Option<&[fret_runtime::WindowHitTestRegionV1]>,
) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get(),
            _ => 0,
        },
        Err(_) => 0,
    };
    if hwnd == 0 {
        return false;
    }
    super::win32::set_window_hit_test_passthrough_regions(hwnd, regions)
}

pub(super) fn set_window_hit_test(
    window: &dyn Window,
    hit_test: &fret_runtime::WindowHitTestRequestV1,
) -> bool {
    use fret_runtime::WindowHitTestRequestV1 as H;

    #[cfg(target_os = "windows")]
    {
        match hit_test {
            H::Normal => {
                let a = set_window_hit_test_passthrough_regions(window, None);
                let b = set_window_hit_test_passthrough_all(window, false);
                a && b
            }
            H::PassthroughAll => {
                let a = set_window_hit_test_passthrough_regions(window, None);
                let b = set_window_hit_test_passthrough_all(window, true);
                a && b
            }
            H::PassthroughRegions { regions } => {
                let a = set_window_hit_test_passthrough_all(window, false);
                let b = set_window_hit_test_passthrough_regions(window, Some(regions));
                a && b
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        match hit_test {
            H::Normal => {
                #[cfg(feature = "macos-hit-test-regions")]
                super::macos_hit_test::clear_passthrough_regions(window);
                set_window_hit_test_passthrough_all(window, false)
            }
            H::PassthroughAll => {
                #[cfg(feature = "macos-hit-test-regions")]
                super::macos_hit_test::clear_passthrough_regions(window);
                set_window_hit_test_passthrough_all(window, true)
            }
            H::PassthroughRegions { regions } => {
                #[cfg(not(feature = "macos-hit-test-regions"))]
                {
                    let _ = regions;
                    // Until region passthrough is stabilized on macOS, fall back to passthrough-all.
                    set_window_hit_test_passthrough_all(window, true)
                }

                #[cfg(feature = "macos-hit-test-regions")]
                {
                    // Best-effort: if regions cannot be installed, fall back to passthrough all.
                    if super::macos_hit_test::set_passthrough_regions(window, regions) {
                        true
                    } else {
                        set_window_hit_test_passthrough_all(window, true)
                    }
                }
            }
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        match hit_test {
            H::Normal => set_window_hit_test_passthrough_all(window, false),
            H::PassthroughAll => set_window_hit_test_passthrough_all(window, true),
            H::PassthroughRegions { .. } => false,
        }
    }
}

#[cfg(target_os = "windows")]
pub(super) fn set_window_background_material(
    window: &dyn Window,
    material: fret_runtime::WindowBackgroundMaterialRequest,
) -> bool {
    use winit::raw_window_handle::HasWindowHandle as _;

    let hwnd: isize = match window.window_handle() {
        Ok(handle) => match handle.as_raw() {
            winit::raw_window_handle::RawWindowHandle::Win32(h) => h.hwnd.get(),
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
pub(super) fn set_window_hit_test_passthrough_all(_window: &dyn Window, _enabled: bool) -> bool {
    false
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
pub(super) fn set_window_background_material(
    _window: &dyn Window,
    _material: fret_runtime::WindowBackgroundMaterialRequest,
) -> bool {
    false
}
