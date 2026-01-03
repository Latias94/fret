use fret_core::Rect;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};

type ImeCursorArea = (i32, i32, i32, i32);
type ImeCursorAreaByHwnd = HashMap<isize, ImeCursorArea>;

static IME_CURSOR_AREA_BY_HWND: LazyLock<Mutex<ImeCursorAreaByHwnd>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
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
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        MSG, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION,
    };

    if msg.is_null() {
        return false;
    }

    if !MSG_HOOK_SEEN.swap(true, Ordering::Relaxed) && ime_debug_enabled() {
        tracing::info!("IME_DEBUG windows_ime: msg_hook installed and receiving messages");
    }

    let msg = msg as *const MSG;
    // SAFETY: winit guarantees this points to a MSG for the duration of the hook call.
    let message = unsafe { (*msg).message };
    if message != WM_IME_STARTCOMPOSITION
        && message != WM_IME_COMPOSITION
        && message != WM_IME_ENDCOMPOSITION
    {
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
    apply_cursor_area_for_hwnd(hwnd);
    false
}

pub fn set_ime_cursor_area(window: &Window, rect: Rect) {
    if !force_imm_enabled() {
        set_ime_cursor_area_via_winit(window, rect);
        return;
    }
    set_ime_cursor_area_via_imm(window, rect);
}

fn hwnd_for_window(window: &Window) -> Option<windows_sys::Win32::Foundation::HWND> {
    let handle = window.window_handle().ok()?;
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return None;
    };
    Some(handle.hwnd.get() as windows_sys::Win32::Foundation::HWND)
}

fn set_ime_cursor_area_via_winit(window: &Window, rect: Rect) {
    let scale_factor = window.scale_factor();
    let x = (rect.origin.x.0 as f64 * scale_factor).round() as i32;
    let y = (rect.origin.y.0 as f64 * scale_factor).round() as i32;
    let width = (rect.size.width.0 as f64 * scale_factor).round().max(1.0) as i32;
    let height = (rect.size.height.0 as f64 * scale_factor).round().max(1.0) as i32;

    // Cache the most recent cursor area so our message hook can apply the composition/candidate
    // position at WM_IME_STARTCOMPOSITION/WM_IME_COMPOSITION time (some IMEs appear to ignore or
    // race winit's update otherwise, defaulting to the client origin).
    if let Some(hwnd) = hwnd_for_window(window) {
        if let Ok(mut map) = IME_CURSOR_AREA_BY_HWND.lock() {
            map.insert(hwnd as isize, (x, y, width, height));
        }
        // Also apply immediately to avoid relying on WM_IME_* timing / message hook delivery.
        apply_cursor_area_for_hwnd(hwnd as isize);
    }

    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: winit set_ime_cursor_area rect=({:.1},{:.1} {:.1}x{:.1}) scale={:.3} -> origin=({}, {}) size=({}x{})",
            rect.origin.x.0,
            rect.origin.y.0,
            rect.size.width.0,
            rect.size.height.0,
            scale_factor,
            x,
            y,
            width,
            height
        );
    }

    window.set_ime_cursor_area(
        winit::dpi::PhysicalPosition::new(x, y),
        winit::dpi::PhysicalSize::new(width.max(1) as u32, height.max(1) as u32),
    );
}

fn set_ime_cursor_area_via_imm(window: &Window, rect: Rect) {
    disable_text_frame_service_if_requested();

    use windows_sys::Win32::Foundation::{POINT, RECT};
    use windows_sys::Win32::UI::Input::Ime::{
        CANDIDATEFORM, CFS_CANDIDATEPOS, CFS_FORCE_POSITION, CFS_POINT, COMPOSITIONFORM,
        ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow, ImmSetCompositionWindow,
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

    if let Ok(mut map) = IME_CURSOR_AREA_BY_HWND.lock() {
        map.insert(hwnd as isize, (x, y, width, height));
    }

    let rc_area = RECT {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    };

    let candidate_form = CANDIDATEFORM {
        dwIndex: 0,
        // Match Zed's behavior: positioning by point tends to be more reliable than EXCLUDE on
        // Microsoft Pinyin (EXCLUDE can cause fallback to a corner in some configurations).
        dwStyle: CFS_CANDIDATEPOS,
        ptCurrentPos: POINT { x, y: spot_y },
        rcArea: rc_area,
    };

    let composition_form = COMPOSITIONFORM {
        // Force the IME to honor the point immediately on first composition. Without this, some
        // configurations appear to ignore our requested position for the initial preedit.
        dwStyle: CFS_POINT | CFS_FORCE_POSITION,
        ptCurrentPos: POINT { x, y: spot_y },
        rcArea: rc_area,
    };

    unsafe {
        let cand_ok = ImmSetCandidateWindow(himc, &candidate_form);
        let comp_ok = ImmSetCompositionWindow(himc, &composition_form);
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

fn apply_cursor_area_for_hwnd(hwnd: isize) {
    disable_text_frame_service_if_requested();

    use windows_sys::Win32::Foundation::{POINT, RECT};
    use windows_sys::Win32::UI::Input::Ime::{
        CANDIDATEFORM, CFS_CANDIDATEPOS, CFS_FORCE_POSITION, CFS_POINT, COMPOSITIONFORM,
        ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow, ImmSetCompositionWindow,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_IMMENABLED};

    if unsafe { GetSystemMetrics(SM_IMMENABLED) } == 0 {
        return;
    }

    let Ok(map) = IME_CURSOR_AREA_BY_HWND.lock() else {
        return;
    };
    let Some((x, y, width, height)) = map.get(&hwnd).copied() else {
        return;
    };
    let spot_y = y + height;

    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return;
    }

    if ime_debug_enabled() {
        tracing::info!(
            "IME_DEBUG windows_ime: msg_hook apply hwnd={:?} origin=({}, {}) size=({}x{})",
            hwnd,
            x,
            y,
            width,
            height
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
        dwStyle: CFS_CANDIDATEPOS,
        ptCurrentPos: POINT { x, y: spot_y },
        rcArea: rc_area,
    };
    let composition_form = COMPOSITIONFORM {
        dwStyle: CFS_POINT | CFS_FORCE_POSITION,
        ptCurrentPos: POINT { x, y: spot_y },
        rcArea: rc_area,
    };

    unsafe {
        let cand_ok = ImmSetCandidateWindow(himc, &candidate_form);
        let comp_ok = ImmSetCompositionWindow(himc, &composition_form);
        if ime_debug_enabled() {
            tracing::info!(
                "IME_DEBUG windows_ime: msg_hook ImmSetCandidateWindow={} ImmSetCompositionWindow={}",
                cand_ok,
                comp_ok
            );
        }
        ImmReleaseContext(hwnd, himc);
    }
}
