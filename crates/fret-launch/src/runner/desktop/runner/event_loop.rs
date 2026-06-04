#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};

use super::{ActiveEventLoop, WinitAppDriver, WinitRunner};
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
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    AssetReloadWake,
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
    #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
    MacosHitTestRefreshRegions,
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let proxy_events = &self.proxy_events;
        let pending = proxy_events
            .lock()
            .ok()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default();

        for event in pending {
            match event {
                RunnerUserEvent::PlatformCompletion { window, completion } => {
                    self.deliver_platform_completion_now(window, completion);
                }
                #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                RunnerUserEvent::AssetReloadWake => {
                    let windows = self.windows.keys().collect::<Vec<_>>();
                    if let Some(asset_reload) = self.asset_reload.as_mut() {
                        let _ = asset_reload.handle_proxy_wake(&mut self.app, &windows);
                    }
                }
                #[cfg(windows)]
                RunnerUserEvent::WindowsMenuCommand { window, command } => {
                    self.app.push_effect(fret_app::Effect::Command {
                        window: Some(window),
                        command,
                    });
                }
                #[cfg(target_os = "macos")]
                RunnerUserEvent::MacosMenuCommand { window, command } => {
                    self.app
                        .push_effect(fret_app::Effect::Command { window, command });
                }
                #[cfg(target_os = "macos")]
                RunnerUserEvent::MacosMenuWillOpen => {
                    super::macos_menu::sync_command_gating_from_app(&self.app);
                }
                #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
                RunnerUserEvent::MacosHitTestRefreshRegions => {
                    super::macos_hit_test::apply_latest_mouse_location();
                }
            }
        }

        self.drain_effects(event_loop);
    }
}
