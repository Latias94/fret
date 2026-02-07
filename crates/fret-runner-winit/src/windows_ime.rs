use fret_core::Rect;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
static MSG_HOOK_SEEN: AtomicBool = AtomicBool::new(false);
static IMM_TSF_DISABLE_ATTEMPTED: OnceLock<()> = OnceLock::new();

fn ime_debug_enabled() -> bool {
    std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty())
}

fn force_imm_enabled() -> bool {
    std::env::var_os("FRET_IME_FORCE_IMM").is_some_and(|v| !v.is_empty())
}

fn disable_text_frame_service_if_requested() {
    if !force_imm_enabled() {
        return;
    }
    if IMM_TSF_DISABLE_ATTEMPTED.set(()).is_err() {
        return;
    }

    use windows_sys::Win32::System::Threading::GetCurrentThreadId;
    use windows_sys::Win32::UI::Input::Ime::ImmDisableTextFrameService;

    let thread_id = unsafe { GetCurrentThreadId() };
    let ok = unsafe { ImmDisableTextFrameService(thread_id) };
    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: ImmDisableTextFrameService(thread_id={}) -> {}",
            thread_id,
            ok
        );
    }
}

pub fn msg_hook(msg: *const c_void) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{MSG, WM_IME_STARTCOMPOSITION};

    if msg.is_null() {
        return false;
    }

    if !MSG_HOOK_SEEN.swap(true, Ordering::Relaxed) && ime_debug_enabled() {
        tracing::info!("IME_DEBUG windows_ime: msg_hook installed and receiving messages");
    }

    let msg = msg as *const MSG;
    // SAFETY: winit guarantees this points to a MSG for the duration of the hook call.
    let message = unsafe { (*msg).message };
    if message != WM_IME_STARTCOMPOSITION {
        return false;
    }

    // SAFETY: see above.
    let hwnd = unsafe { (*msg).hwnd } as isize;
    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: msg_hook message=0x{:04X} hwnd={}",
            message,
            hwnd
        );
    }

    // No-op: `request_ime_update` already updates the IME cursor area on the winit event thread.
    // Avoid calling into IMM from within the message hook, which can create re-entrant WM_IME_*
    // loops and hangs depending on the active IME.
    false
}

pub fn set_ime_cursor_area(window: &dyn Window, rect: Rect) {
    if !force_imm_enabled() {
        set_ime_cursor_area_via_winit(window, rect);
        return;
    }
    set_ime_cursor_area_via_imm(window, rect);
}

fn hwnd_for_window(window: &dyn Window) -> Option<windows_sys::Win32::Foundation::HWND> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    Some(handle.hwnd.get() as windows_sys::Win32::Foundation::HWND)
}

fn set_ime_cursor_area_via_winit(window: &dyn Window, rect: Rect) {
    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: winit set_ime_cursor_area rect=({:.1},{:.1} {:.1}x{:.1})",
            rect.origin.x.0,
            rect.origin.y.0,
            rect.size.width.0,
            rect.size.height.0,
        );
    }

    // Forward logical coordinates and let winit apply per-backend conversion rules.
    let request_data = winit::window::ImeRequestData::default().with_cursor_area(
        winit::dpi::LogicalPosition::new(rect.origin.x.0, rect.origin.y.0).into(),
        winit::dpi::LogicalSize::new(rect.size.width.0.max(1.0), rect.size.height.0.max(1.0))
            .into(),
    );
    let _ = window.request_ime_update(winit::window::ImeRequest::Update(request_data));
}

fn set_ime_cursor_area_via_imm(window: &dyn Window, rect: Rect) {
    disable_text_frame_service_if_requested();

    use windows_sys::Win32::Foundation::{POINT, RECT};
    use windows_sys::Win32::UI::Input::Ime::{
        CANDIDATEFORM, CFS_EXCLUDE, CFS_POINT, COMPOSITIONFORM, ImmGetContext, ImmReleaseContext,
        ImmSetCandidateWindow, ImmSetCompositionWindow,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_IMMENABLED};

    if unsafe { GetSystemMetrics(SM_IMMENABLED) } == 0 {
        return;
    }

    let Some(hwnd) = hwnd_for_window(window) else {
        if ime_debug_enabled() {
            tracing::info!("IME_DEBUG windows_ime: not a Win32 window handle");
        }
        return;
    };
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        if ime_debug_enabled() {
            tracing::info!("IME_DEBUG windows_ime: ImmGetContext returned null");
        }
        return;
    }

    let scale_factor = window.scale_factor();
    let x = (rect.origin.x.0 as f64 * scale_factor).round() as i32;
    let y = (rect.origin.y.0 as f64 * scale_factor).round() as i32;
    let width = (rect.size.width.0 as f64 * scale_factor).round().max(1.0) as i32;
    let height = (rect.size.height.0 as f64 * scale_factor).round().max(1.0) as i32;
    let spot_y = y + height;

    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: imm set_cursor_area rect=({:.1},{:.1} {:.1}x{:.1}) scale={:.3} -> origin=({}, {}) size=({}x{}) spot=({}, {})",
            rect.origin.x.0,
            rect.origin.y.0,
            rect.size.width.0,
            rect.size.height.0,
            scale_factor,
            x,
            y,
            width,
            height,
            x,
            spot_y
        );
    }

    let rc_area = RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };

    let candidate_form = CANDIDATEFORM {
        dwIndex: 0,
        dwStyle: CFS_EXCLUDE,
        ptCurrentPos: POINT { x, y },
        rcArea: rc_area,
    };

    let composition_form = COMPOSITIONFORM {
        dwStyle: CFS_POINT,
        ptCurrentPos: POINT { x, y: spot_y },
        rcArea: rc_area,
    };

    unsafe {
        let comp_ok = ImmSetCompositionWindow(himc, &composition_form);
        let cand_ok = ImmSetCandidateWindow(himc, &candidate_form);
        if ime_debug_enabled() {
            tracing::info!(
                "IME_DEBUG windows_ime: ImmSetCandidateWindow={} ImmSetCompositionWindow={}",
                cand_ok,
                comp_ok
            );
        }
        ImmReleaseContext(hwnd, himc);
    }
}
