use fret_core::WindowMetricsService;
use fret_core::time::Instant;
use winit::event_loop::ActiveEventLoop;

#[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
use super::macos_hit_test;
#[cfg(target_os = "macos")]
use super::macos_menu;
#[cfg(windows)]
use super::windows_menu;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_close_request(
        &mut self,
        window: fret_core::AppWindowId,
        event_loop: &dyn ActiveEventLoop,
    ) -> bool {
        let is_main = Some(window) == self.main_window;
        if !self.close_window(window) {
            return false;
        }

        if is_main && self.config.exit_on_main_window_close {
            self.force_close_all_windows();
            self.shutdown_event_loop(event_loop);
            return true;
        }

        if self.windows.is_empty() {
            self.shutdown_event_loop(event_loop);
            return true;
        }

        false
    }

    fn force_close_all_windows(&mut self) {
        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
        for window in windows {
            let _ = self.force_close_window(window);
        }
    }

    fn shutdown_event_loop(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.dispatcher.shutdown();
        event_loop.exit();
    }

    pub(super) fn close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, true)
    }

    pub(super) fn force_close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, false)
    }

    fn close_window_impl(
        &mut self,
        window: fret_core::AppWindowId,
        check_before_close: bool,
    ) -> bool {
        if !self.windows.contains_key(window) {
            return false;
        }

        if check_before_close {
            let should_close = self.driver.before_close_window(&mut self.app, window);
            if !should_close {
                return false;
            }
        }

        #[cfg(feature = "dev-state")]
        if check_before_close && self.dev_state.enabled() {
            let alive: std::collections::HashSet<fret_core::AppWindowId> =
                self.windows.keys().collect();
            self.dev_state
                .sync_window_keys_from_app(&self.app, |window| alive.contains(&window));

            let key = self.dev_state.window_key(window).map(ToString::to_string);
            if let Some(key) = key
                && let Some(state) = self.windows.get(window)
            {
                let physical = state.window.surface_size();
                let logical: winit::dpi::LogicalSize<f64> =
                    physical.to_logical(state.window.scale_factor());
                let position = state.window.outer_position().ok();
                self.dev_state
                    .observe_window_geometry_now(&key, logical, position);
            }

            self.dev_state.export_and_flush_now(&mut self.app);
        }

        if self
            .dock_tearoff_follow
            .is_some_and(|f| f.window == window || f.source_window == window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }
        self.dock_floating_windows.remove(&window);

        if self.internal_drag_hover_window == Some(window) {
            self.internal_drag_hover_window = None;
            self.internal_drag_hover_pos = None;
            self.internal_drag_pointer_id = None;
        }

        {
            use fret_runtime::DragHost as _;
            use std::collections::HashSet;

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.source_window == window
            }) {
                visited.insert(pointer_id);
                self.app.cancel_drag(pointer_id);
            }

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.current_window == window
            }) {
                visited.insert(pointer_id);
                if let Some(drag) = self.app.drag_mut(pointer_id) {
                    drag.current_window = drag.source_window;
                }
            }
        }

        self.webviews.close_window(&mut self.app, window);

        let Some(state) = self.windows.remove(window) else {
            return false;
        };
        #[cfg(feature = "dev-state")]
        self.dev_state.unregister_window(window);
        self.windows_z_order.retain(|w| *w != window);
        #[cfg(windows)]
        windows_menu::unregister_window(state.window.as_ref());
        #[cfg(target_os = "macos")]
        macos_menu::unregister_window(state.window.as_ref());
        #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
        macos_hit_test::unregister_window(state.window.as_ref());
        self.window_registry.remove(state.window.id());
        self.app.with_global_mut(
            fret_runtime::RunnerWindowLifecycleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_close(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_close(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerPresentDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerFrameDriveDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::WindowRedrawRequestDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::WindowGlobalChangeDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerSurfaceConfigDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );

        self.app.with_global_mut(
            fret_runtime::WindowInputContextService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandEnabledService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowTextInputSnapshotService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                svc.remove(window);
            });
        if Some(window) == self.main_window {
            self.main_window = None;
        }

        true
    }
}
