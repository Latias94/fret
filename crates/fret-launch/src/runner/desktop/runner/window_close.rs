use winit::event_loop::ActiveEventLoop;

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
}
