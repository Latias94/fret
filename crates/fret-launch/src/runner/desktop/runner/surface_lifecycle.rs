use fret_core::AppWindowId;
use tracing::error;

use super::{SurfaceState, WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    fn surface_usage_for_new_surface(&self) -> wgpu::TextureUsages {
        let base = self.diag_bundle_screenshots.surface_usage();
        #[cfg(feature = "diag-screenshots")]
        {
            if self.diag_screenshots.is_some() {
                base | wgpu::TextureUsages::COPY_SRC
            } else {
                base
            }
        }
        #[cfg(not(feature = "diag-screenshots"))]
        {
            base
        }
    }

    pub(super) fn try_create_missing_surfaces(&mut self) {
        let Some(context) = self.context.as_ref() else {
            return;
        };

        let surface_usage = self.surface_usage_for_new_surface();
        let mut redraw_bootstrap_windows: Vec<AppWindowId> = Vec::new();
        for (app_window, state) in self.windows.iter_mut() {
            if state.surface.is_some() {
                continue;
            }

            let surface = match context.create_surface(state.window.clone()) {
                Ok(surface) => surface,
                Err(e) => {
                    error!(window = ?app_window, error = ?e, "failed to create surface");
                    continue;
                }
            };

            let size = state.window.surface_size();
            let mut surface_state = match SurfaceState::new_with_usage(
                &context.adapter,
                &context.device,
                surface,
                size.width,
                size.height,
                surface_usage,
            ) {
                Ok(state) => state,
                Err(e) => {
                    error!(
                        window = ?app_window,
                        error = ?e,
                        "failed to configure surface"
                    );
                    continue;
                }
            };

            let want_surface_composited_alpha = self
                .app
                .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>()
                .and_then(|s| s.effective_snapshot(app_window))
                .is_some_and(|s| s.surface_composited_alpha);
            super::window_lifecycle::configure_surface_alpha_mode_for_composited_window(
                &context.adapter,
                &context.device,
                &mut surface_state,
                want_surface_composited_alpha,
            );

            state.surface = Some(surface_state);
            redraw_bootstrap_windows.push(app_window);
        }

        for app_window in redraw_bootstrap_windows {
            let _ = self.request_window_redraw_with_reason(
                app_window,
                fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,
            );
            // Match the normal window creation bootstrap: a raw redraw request may not wake the
            // event loop on every platform, so deferred surface creation also gets a one-shot RAF.
            self.raf_windows.request(app_window);
        }
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(super) fn attach_factory_surface_to_main_window(
        &mut self,
        main_window: AppWindowId,
        surface: wgpu::Surface<'static>,
    ) -> bool {
        let Some(context) = self.context.as_ref() else {
            return true;
        };
        let surface_usage = self.surface_usage_for_new_surface();
        let Some(state) = self.windows.get_mut(main_window) else {
            return true;
        };

        let size = state.window.surface_size();
        let mut surface_state = match SurfaceState::new_with_usage(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
            surface_usage,
        ) {
            Ok(v) => v,
            Err(e) => {
                error!(
                    window = ?main_window,
                    error = ?e,
                    "failed to configure factory surface"
                );
                return false;
            }
        };

        let want_surface_composited_alpha = self
            .app
            .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>()
            .and_then(|s| s.effective_snapshot(main_window))
            .is_some_and(|s| s.surface_composited_alpha);
        super::window_lifecycle::configure_surface_alpha_mode_for_composited_window(
            &context.adapter,
            &context.device,
            &mut surface_state,
            want_surface_composited_alpha,
        );
        state.surface = Some(surface_state);
        true
    }

    pub(super) fn destroy_runner_surfaces(&mut self) {
        for (_app_window, state) in self.windows.iter_mut() {
            state.surface = None;
            state.pending_surface_resize = None;
        }
        self.raf_windows.clear();
        self.next_raf_deadline = None;
    }
}
