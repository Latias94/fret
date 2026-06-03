use winit::event_loop::ActiveEventLoop;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_quit_app_effect(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        let prompt_window = self.main_window.or_else(|| self.windows.keys().next());
        if let Some(window) = prompt_window
            && !self.driver.before_close_window(&mut self.app, window)
        {
            return false;
        }

        #[cfg(feature = "dev-state")]
        if self.dev_state.enabled() {
            self.flush_dev_state_for_quit_app_effect();
        }

        let windows: Vec<fret_core::AppWindowId> = self.windows.keys().collect();
        for window in windows {
            let _ = self.force_close_window(window);
        }

        self.dispatcher.shutdown();
        event_loop.exit();
        true
    }

    #[cfg(feature = "dev-state")]
    fn flush_dev_state_for_quit_app_effect(&mut self) {
        let alive: std::collections::HashSet<fret_core::AppWindowId> =
            self.windows.keys().collect();
        self.dev_state
            .sync_window_keys_from_app(&self.app, |window| alive.contains(&window));

        let keys = self.dev_state.window_keys_snapshot();
        for (window, key) in keys {
            let Some(state) = self.windows.get(window) else {
                continue;
            };
            let physical = state.window.surface_size();
            let logical: winit::dpi::LogicalSize<f64> =
                physical.to_logical(state.window.scale_factor());
            let position = state.window.outer_position().ok();
            self.dev_state
                .observe_window_geometry_now(&key, logical, position);
        }
        self.dev_state.export_and_flush_now(&mut self.app);
    }
}
