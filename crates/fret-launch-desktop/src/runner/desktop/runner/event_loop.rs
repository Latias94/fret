#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};

use fret_runtime::PlatformCompletion;

#[cfg(target_os = "macos")]
use fret_runtime::CommandId as RuntimeCommandId;

#[cfg(windows)]
static WINDOWS_IME_MSG_HOOK_ENABLED: AtomicBool = AtomicBool::new(true);

#[cfg(windows)]
pub(super) fn set_windows_ime_msg_hook_enabled(enabled: bool) {
    WINDOWS_IME_MSG_HOOK_ENABLED.store(enabled, Ordering::Relaxed);
}

#[cfg(windows)]
pub fn windows_msg_hook(msg: *const std::ffi::c_void) -> bool {
    if WINDOWS_IME_MSG_HOOK_ENABLED.load(Ordering::Relaxed) {
        fret_runner_winit::windows_ime::msg_hook(msg);
    }
    super::windows_menu::msg_hook(msg)
}

#[derive(Debug, Clone)]
pub enum RunnerUserEvent {
    PlatformCompletion {
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    },
    #[cfg(windows)]
    WindowsMenuCommand {
        window: fret_core::AppWindowId,
        command: fret_runtime::CommandId,
    },
    #[cfg(target_os = "macos")]
    MacosMenuCommand {
        window: Option<fret_core::AppWindowId>,
        command: RuntimeCommandId,
    },
    #[cfg(target_os = "macos")]
    MacosMenuWillOpen,
}
