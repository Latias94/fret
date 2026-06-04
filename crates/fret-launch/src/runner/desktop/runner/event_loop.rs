#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};

use super::{ActiveEventLoop, WinitAppDriver, WinitRunner};
use fret_core::time::{Duration, Instant};
use fret_runtime::PlatformCompletion;
use winit::event_loop::ControlFlow;

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

    pub(super) fn handle_about_to_wait_preamble(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        // Ensure effects requested during `RedrawRequested` (after the pre-render drain) are still
        // observed before the loop sleeps (e.g. `App::request_redraw()` inside a render callback).
        self.drain_effects(event_loop);

        if self.is_suspended {
            event_loop.set_control_flow(ControlFlow::Wait);
            return true;
        }

        self.refresh_runner_monitor_topology_diagnostics(event_loop);
        false
    }

    pub(super) fn handle_about_to_wait_turn_bookkeeping(&mut self) -> Instant {
        self.tick_id = super::scheduling::begin_turn(&mut self.tick_id);
        self.app.set_tick_id(self.tick_id);
        self.saw_left_mouse_release_this_turn = false;
        let now = Instant::now();
        self.poll_window_environment_if_due(now);
        now
    }

    pub(super) fn handle_about_to_wait_control_flow(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        now: Instant,
    ) {
        let did_pending_front_work = self.process_pending_front_requests(now);

        let mut next_deadline: Option<Instant> = None;
        for entry in self.timers.values() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(entry.deadline),
                None => entry.deadline,
            });
        }

        if let Some(deadline) = self.dispatcher.next_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        if let Some(deadline) = self.next_pending_front_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        #[cfg(feature = "hotpatch-subsecond")]
        if let Some(trigger) = self.hotpatch.as_ref() {
            if let Some(deadline) = trigger.next_poll_at() {
                next_deadline = Some(match next_deadline {
                    Some(cur) => cur.min(deadline),
                    None => deadline,
                });
            }
        }

        let drag_poll = self.dock_drag_pointer_id().is_some();
        let follow_poll = self.dock_tearoff_follow.is_some();
        let wants_poll = drag_poll || follow_poll;

        let raf_deadline = if self.raf_windows.has_pending() {
            let deadline = *self
                .next_raf_deadline
                .get_or_insert_with(|| now + self.config.frame_interval);
            Some(deadline)
        } else {
            self.next_raf_deadline = None;
            None
        };
        let flushed_raf_this_turn = raf_deadline.is_some_and(|deadline| now >= deadline);
        if flushed_raf_this_turn {
            self.next_raf_deadline = None;
            self.flush_raf_redraw_requests();
        }

        let next = match (
            next_deadline,
            raf_deadline.filter(|deadline| now < *deadline),
        ) {
            (Some(deadline), Some(raf_deadline)) => Some(deadline.min(raf_deadline)),
            (Some(deadline), None) => Some(deadline),
            (None, Some(raf_deadline)) => Some(raf_deadline),
            (None, None) => None,
        };

        if wants_poll || flushed_raf_this_turn {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else if let Some(next) = next {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next));
        } else if did_pending_front_work {
            // Ensure we keep turning the event loop while we try to raise a window on macOS.
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(16)));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
