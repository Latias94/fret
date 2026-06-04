use fret_core::AppWindowId;
use tracing::error;

#[cfg(any(target_os = "android", target_os = "ios"))]
use super::ActiveEventLoop;
use super::{PlatformCapabilities, SurfaceState, WindowStyleRequest, WinitAppDriver, WinitRunner};
#[cfg(any(target_os = "android", target_os = "ios"))]
use winit::event_loop::ControlFlow;

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
            configure_surface_alpha_mode_for_composited_window(
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
        configure_surface_alpha_mode_for_composited_window(
            &context.adapter,
            &context.device,
            &mut surface_state,
            want_surface_composited_alpha,
        );
        state.surface = Some(surface_state);
        true
    }

    pub(super) fn handle_destroy_surfaces(&mut self) {
        self.app.with_global_mut(
            fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore::default,
            |store, _app| {
                store.record_destroy_surfaces();
            },
        );

        self.destroy_runner_surfaces();
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(super) fn handle_about_to_wait_mobile_surface_recreation(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        // Only attempt to (re)create missing surfaces after winit has indicated surfaces may be
        // created for this lifecycle turn. Calling the `can_create_surfaces` hook directly would
        // bypass the winit gate and can fail early on Android.
        let surfaces_available = self
            .app
            .global::<fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore>()
            .map(|s| s.snapshot().surfaces_available)
            .unwrap_or(false);

        if surfaces_available && self.context.is_some() {
            let needs_surfaces = self
                .windows
                .iter()
                .any(|(_app_window, state)| state.surface.is_none());
            if needs_surfaces {
                self.try_create_missing_surfaces();
                self.drain_effects(event_loop);
            }
        }
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(super) fn handle_resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.is_suspended = false;

        for (app_window, state) in self.windows.iter() {
            let _ = (app_window, state);
            state.window.request_redraw();
        }
        self.drain_effects(event_loop);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    pub(super) fn handle_suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.is_suspended = true;

        // Best-effort: drop surfaces to avoid presenting while backgrounded and to ensure we can
        // recreate cleanly on resume.
        self.handle_destroy_surfaces();
        event_loop.set_control_flow(ControlFlow::Wait);
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

pub(super) fn want_surface_composited_alpha_for_style(
    style: WindowStyleRequest,
    caps: &PlatformCapabilities,
) -> bool {
    if !caps.ui.window_transparent {
        return false;
    }

    if let Some(transparent) = style.transparent {
        return transparent;
    }

    if let Some(material) = style.background_material {
        let clamped = fret_runtime::clamp_background_material_request(material, caps);
        if clamped != fret_runtime::WindowBackgroundMaterialRequest::None {
            // Background materials may require a composited alpha surface (ADR 0310). If the
            // caller did not explicitly request `transparent`, treat it as implied once a
            // non-None material is effectively applied.
            return true;
        }
    }

    false
}

pub(super) fn configure_surface_alpha_mode_for_composited_window(
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    surface: &mut SurfaceState<'_>,
    want_surface_composited_alpha: bool,
) {
    let capabilities = surface.surface.get_capabilities(adapter);
    if capabilities.alpha_modes.is_empty() {
        return;
    }

    let desired = if want_surface_composited_alpha {
        // Prefer explicit alpha composition modes over `Opaque` when we want a composited window.
        // Ordering is "best-effort" and may vary by platform/backend.
        [
            wgpu::CompositeAlphaMode::PreMultiplied,
            wgpu::CompositeAlphaMode::PostMultiplied,
            wgpu::CompositeAlphaMode::Inherit,
            // `Auto` may pick an opaque path even for transparent windows on some backends.
            // Prefer `Inherit` first so the platform can select the appropriate compositing mode.
            wgpu::CompositeAlphaMode::Auto,
        ]
        .into_iter()
        .find(|m| capabilities.alpha_modes.contains(m))
        .unwrap_or(capabilities.alpha_modes[0])
    } else {
        capabilities
            .alpha_modes
            .iter()
            .copied()
            .find(|m| matches!(m, wgpu::CompositeAlphaMode::Opaque))
            .unwrap_or(capabilities.alpha_modes[0])
    };

    if surface.config.alpha_mode != desired {
        surface.config.alpha_mode = desired;
        if let Err(err) = surface.reconfigure(device) {
            tracing::error!(
                error = ?err,
                "failed to reconfigure surface alpha mode for composited window"
            );
        }
    }
}
