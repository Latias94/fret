use fret_core::AppWindowId;
use fret_render::{RenderError, SurfaceAcquireError};
use fret_runtime::RunnerFrameDriveReason;
use tracing::error;
use winit::event_loop::ActiveEventLoop;

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_redraw_present_error(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        err: RenderError,
    ) {
        match err {
            RenderError::SurfaceAcquireFailed {
                source: SurfaceAcquireError::Lost,
            } => {
                self.clear_window_surface_after_present_acquire_failure(app_window);
                let _ = self.request_window_redraw_with_reason(
                    app_window,
                    RunnerFrameDriveReason::SurfaceRecoverLost,
                );
                self.raf_windows.request(app_window);
            }
            RenderError::SurfaceAcquireFailed {
                source: SurfaceAcquireError::Outdated,
            } => {
                self.clear_window_surface_after_present_acquire_failure(app_window);
                let _ = self.request_window_redraw_with_reason(
                    app_window,
                    RunnerFrameDriveReason::SurfaceRecoverOutdated,
                );
                self.raf_windows.request(app_window);
            }
            RenderError::SurfaceAcquireFailed {
                source: SurfaceAcquireError::Timeout,
            } => {
                // Transient on some platforms during startup/resize; request one more redraw so
                // the window does not stay blank until the next user input.
                let _ = self.request_window_redraw_with_reason(
                    app_window,
                    RunnerFrameDriveReason::SurfaceRecoverTimeout,
                );
                self.raf_windows.request(app_window);
            }
            RenderError::SurfaceAcquireFailed {
                source: SurfaceAcquireError::OutOfMemory,
            } => {
                self.dispatcher.shutdown();
                event_loop.exit();
            }
            RenderError::SurfaceAcquireFailed { .. } => {}
            _ => {
                error!(?err, "render error");
            }
        }
    }

    fn clear_window_surface_after_present_acquire_failure(&mut self, app_window: AppWindowId) {
        if let Some(state) = self.windows.get_mut(app_window) {
            state.surface = None;
        }
    }
}
