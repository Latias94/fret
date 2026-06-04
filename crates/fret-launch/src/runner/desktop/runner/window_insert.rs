use std::sync::Arc;

use fret_core::{AppWindowId, Event, Px, Scene};
use fret_render::SurfaceState;
use fret_runner_winit::accessibility;
use fret_runtime::{PlatformCapabilities, WindowStyleRequest};
use winit::window::Window;

#[cfg(target_os = "macos")]
use super::macos_menu;
use super::window::WindowRuntime;
#[cfg(windows)]
use super::windows_menu;
use super::{WinitAppDriver, WinitRunner};
use crate::RunnerError;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn insert_window(
        &mut self,
        window: Arc<dyn Window>,
        accessibility: Option<accessibility::WinitAccessibility>,
        surface: Option<wgpu::Surface<'static>>,
        style: WindowStyleRequest,
    ) -> Result<AppWindowId, RunnerError> {
        let surface = if let Some(surface) = surface {
            let Some(context) = self.context.as_ref() else {
                return Err(RunnerError::WgpuNotInitialized);
            };

            let size = window.surface_size();
            let surface_usage = {
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
            };
            let mut state = SurfaceState::new_with_usage(
                &context.adapter,
                &context.device,
                surface,
                size.width,
                size.height,
                surface_usage,
            )?;

            let caps = self
                .app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let want_surface_composited_alpha =
                super::surface_lifecycle::want_surface_composited_alpha_for_style(style, &caps);
            super::surface_lifecycle::configure_surface_alpha_mode_for_composited_window(
                &context.adapter,
                &context.device,
                &mut state,
                want_surface_composited_alpha,
            );

            Some(state)
        } else {
            None
        };

        let id = self.windows.insert_with_key(|id| {
            let user = self.driver.create_window_state(&mut self.app, id);
            WindowRuntime {
                window,
                accessibility,
                last_semantics_snapshot: None,
                surface,
                scene: Scene::default(),
                platform: fret_runner_winit::WinitPlatform {
                    wheel: fret_runner_winit::WheelConfig {
                        line_delta_px: self.config.wheel_line_delta_px,
                        pixel_delta_scale: self.config.wheel_pixel_delta_scale,
                    },
                    ..Default::default()
                },
                pending_wheel: None,
                #[cfg(target_os = "android")]
                android_bottom_inset_baseline: None,
                pending_surface_resize: None,
                last_delivered_window_resized: None,
                is_focused: false,
                external_drag_files: Vec::new(),
                external_drag_token: None,
                user,
                #[cfg(windows)]
                os_menu: None,
            }
        });

        if let Some(state) = self.windows.get(id) {
            let size_phys = state.window.surface_size();
            let size_logical: winit::dpi::LogicalSize<f32> =
                size_phys.to_logical(state.window.scale_factor());
            fret_runtime::apply_window_metrics_event(
                &mut self.app,
                id,
                &Event::WindowResized {
                    width: Px(size_logical.width),
                    height: Px(size_logical.height),
                },
            );
            fret_runtime::apply_window_metrics_event(
                &mut self.app,
                id,
                &Event::WindowScaleFactorChanged(state.window.scale_factor() as f32),
            );
            let surface_record = state.surface.as_ref().map(|surface| {
                super::render::capture_surface_config_diagnostics_record(&surface.config)
            });
            let _ = state;
            if let Some(surface_record) = surface_record {
                self.record_surface_config_snapshot(id, surface_record);
            }
        }
        let window_ref = self.windows.get(id).map(|s| s.window.clone());
        if let Some(window_ref) = window_ref
            && self.update_window_environment_for_window_ref(id, window_ref.as_ref())
        {
            self.app.request_redraw(id);
        }

        let winit_id = self.windows[id].window.id();
        self.window_registry.insert(winit_id, id);
        self.bump_window_z_order(id);
        self.app.with_global_mut(
            fret_runtime::RunnerWindowLifecycleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_open(id);
            },
        );

        #[cfg(windows)]
        windows_menu::register_window(self.windows[id].window.as_ref(), id);
        #[cfg(target_os = "macos")]
        macos_menu::register_window(self.windows[id].window.as_ref(), id);

        #[cfg(windows)]
        if let Some(menu_bar) = self.menu_bar.as_ref()
            && let Some(state) = self.windows.get_mut(id)
            && let Some(menu) =
                windows_menu::set_window_menu_bar(&self.app, state.window.as_ref(), id, menu_bar)
        {
            state.os_menu = Some(menu);
        }

        // Ensure the window draws at least one frame after creation.
        //
        // Important: `WindowEvent::RedrawRequested` is keyed by the winit `WindowId`, so we must
        // install the `WindowId` -> `AppWindowId` mapping before requesting the redraw. Otherwise,
        // the first redraw can be dropped and the window may appear blank until another event.
        if self.windows.contains_key(id) {
            let _ = self.request_window_redraw_with_reason(
                id,
                fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,
            );
            // `request_redraw()` alone may not wake the event loop on some platforms; schedule a
            // one-shot RAF so the initial frame presents without requiring any user input.
            self.raf_windows.request(id);
        }
        Ok(id)
    }
}
