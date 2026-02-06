use fret_core::Rect;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

use std::cell::Cell;
use std::collections::HashMap;
use std::ffi::c_void;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};

type ImeCursorArea = (i32, i32, i32, i32);
type ImeCursorAreaByHwnd = HashMap<isize, ImeCursorArea>;

static IME_CURSOR_AREA_BY_HWND: LazyLock<Mutex<ImeCursorAreaByHwnd>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static MSG_HOOK_SEEN: AtomicBool = AtomicBool::new(false);
static IMM_TSF_DISABLE_ATTEMPTED: OnceLock<()> = OnceLock::new();

#[cfg(test)]
static MSG_HOOK_APPLY_CALLS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static APPLY_CURSOR_AREA_IN_PROGRESS: Cell<bool> = const { Cell::new(false) };
}

struct ApplyCursorAreaGuard;

impl ApplyCursorAreaGuard {
    fn try_enter() -> Option<Self> {
        APPLY_CURSOR_AREA_IN_PROGRESS.with(|flag| {
            if flag.get() {
                None
            } else {
                flag.set(true);
                Some(Self)
            }
        })
    }
}

impl Drop for ApplyCursorAreaGuard {
    fn drop(&mut self) {
        APPLY_CURSOR_AREA_IN_PROGRESS.with(|flag| flag.set(false));
    }
}

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

    // Apply only on composition start. Applying on every composition update can cause re-entrant
    // hook loops and hangs on some IMEs.
    #[cfg(test)]
    MSG_HOOK_APPLY_CALLS.fetch_add(1, Ordering::Relaxed);
    apply_cursor_area_for_hwnd(hwnd);
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
    let scale_factor = window.scale_factor();
    let x = (rect.origin.x.0 as f64 * scale_factor).round() as i32;
    let y = (rect.origin.y.0 as f64 * scale_factor).round() as i32;
    let width = (rect.size.width.0 as f64 * scale_factor).round().max(1.0) as i32;
    let height = (rect.size.height.0 as f64 * scale_factor).round().max(1.0) as i32;

    if let Some(hwnd) = hwnd_for_window(window) {
        if let Ok(mut map) = IME_CURSOR_AREA_BY_HWND.lock() {
            map.insert(hwnd as isize, (x, y, width, height));
        }
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

    let request_data = winit::window::ImeRequestData::default().with_cursor_area(
        winit::dpi::PhysicalPosition::new(x, y).into(),
        winit::dpi::PhysicalSize::new(width.max(1) as u32, height.max(1) as u32).into(),
    );
    let _ = window.request_ime_update(winit::window::ImeRequest::Update(request_data));
}

fn set_ime_cursor_area_via_imm(window: &dyn Window, rect: Rect) {
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

    // Avoid re-entrancy: some IME operations can synchronously send WM_IME_* messages, which can
    // re-enter our message hook. Applying the cursor area again while we're already doing so can
    // cause unbounded recursion or hangs.
    let Some(_guard) = ApplyCursorAreaGuard::try_enter() else {
        return;
    };

    use windows_sys::Win32::Foundation::{POINT, RECT};
    use windows_sys::Win32::UI::Input::Ime::{
        CANDIDATEFORM, CFS_CANDIDATEPOS, CFS_FORCE_POSITION, CFS_POINT, COMPOSITIONFORM,
        ImmGetContext, ImmReleaseContext, ImmSetCandidateWindow, ImmSetCompositionWindow,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_IMMENABLED};

    if unsafe { GetSystemMetrics(SM_IMMENABLED) } == 0 {
        return;
    }

    // IMPORTANT: do not hold the map lock while calling into IMM/TSF.
    //
    // Some IME operations may synchronously send WM_IME_* messages, which can re-enter
    // our message hook and attempt to re-lock this mutex. Holding the lock across the
    // Win32 calls would deadlock.
    let (x, y, width, height) = {
        let Ok(map) = IME_CURSOR_AREA_BY_HWND.lock() else {
            return;
        };
        let Some(rect) = map.get(&hwnd).copied() else {
            return;
        };
        rect
    };
    let spot_y = y + height;

    let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_null() {
        return;
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
        let _ = ImmSetCandidateWindow(himc, &candidate_form);
        let _ = ImmSetCompositionWindow(himc, &composition_form);
        ImmReleaseContext(hwnd, himc);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_cursor_area_guard_prevents_reentrancy() {
        let guard = ApplyCursorAreaGuard::try_enter().expect("first enter should succeed");
        assert!(
            ApplyCursorAreaGuard::try_enter().is_none(),
            "nested enter should be rejected"
        );
        drop(guard);
        assert!(
            ApplyCursorAreaGuard::try_enter().is_some(),
            "enter should succeed again after drop"
        );
    }

    #[test]
    fn msg_hook_applies_only_on_wm_ime_startcomposition() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            MSG, WM_IME_COMPOSITION, WM_IME_ENDCOMPOSITION, WM_IME_STARTCOMPOSITION,
        };

        let mut msg: MSG = unsafe { std::mem::zeroed() };

        MSG_HOOK_APPLY_CALLS.store(0, Ordering::Relaxed);

        msg.message = WM_IME_COMPOSITION;
        let _ = msg_hook((&msg as *const MSG).cast::<c_void>());
        assert_eq!(MSG_HOOK_APPLY_CALLS.load(Ordering::Relaxed), 0);

        msg.message = WM_IME_ENDCOMPOSITION;
        let _ = msg_hook((&msg as *const MSG).cast::<c_void>());
        assert_eq!(MSG_HOOK_APPLY_CALLS.load(Ordering::Relaxed), 0);

        msg.message = WM_IME_STARTCOMPOSITION;
        let _ = msg_hook((&msg as *const MSG).cast::<c_void>());
        assert_eq!(MSG_HOOK_APPLY_CALLS.load(Ordering::Relaxed), 1);
    }
}
