use fret_core::{AppWindowId, Event};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_effect_redraw(&mut self, window: AppWindowId) {
        if self.request_window_redraw_with_reason(
            window,
            fret_runtime::RunnerFrameDriveReason::EffectRedraw,
        ) {
            // Some platforms may not wake the event loop for `request_redraw()` alone; scheduling
            // a one-shot RAF ensures the first frame presents without requiring any input events.
            self.raf_windows.request(window);
        }
    }

    pub(super) fn handle_request_animation_frame(&mut self, window: AppWindowId) {
        self.raf_windows.request(window);
        if self.windows.contains_key(window) {
            self.record_frame_drive_reason(
                window,
                fret_runtime::RunnerFrameDriveReason::EffectRequestAnimationFrame,
            );
        }
    }

    pub(super) fn handle_diag_inject_event(&mut self, window: AppWindowId, event: Event) {
        fret_runtime::with_injected_event_scope(|| {
            self.deliver_window_event_now(window, &event);
        });
        if self.windows.contains_key(window) {
            let _ = self.request_window_redraw_with_reason(
                window,
                fret_runtime::RunnerFrameDriveReason::EffectRedraw,
            );
            self.raf_windows.request(window);
        }
    }
}
