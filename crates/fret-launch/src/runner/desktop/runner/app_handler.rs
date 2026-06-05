//! Winit `ApplicationHandler` integration.

use super::*;

use fret_core::time::Instant;
#[cfg(target_os = "macos")]
use objc2_metal::MTLDevice as _;

impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {
    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<winit::event::DeviceId>,
        event: DeviceEvent,
    ) {
        self.handle_device_event(event_loop, event);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.app.with_global_mut(
            fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore::default,
            |store, _app| {
                store.record_can_create_surfaces();
            },
        );

        if self.wgpu_init_blocked {
            return;
        }

        if self.context.is_some() {
            self.try_create_missing_surfaces();
            self.drain_effects(event_loop);
            return;
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            if self.main_window.is_none() {
                #[cfg(feature = "dev-state")]
                let spec = {
                    let mut spec = self.config.main_window_spec();
                    self.dev_state.apply_main_window_spec(&mut spec);
                    spec
                };
                #[cfg(not(feature = "dev-state"))]
                let spec = self.config.main_window_spec();
                let style = self.config.main_window_style.clone();
                let caps = self
                    .app
                    .global::<fret_runtime::PlatformCapabilities>()
                    .cloned()
                    .unwrap_or_default();
                let window =
                    match self.create_os_window(event_loop, spec, style.clone(), None, &caps) {
                        Ok(w) => w,
                        Err(e) => {
                            error!(error = ?e, "failed to create main window");
                            return;
                        }
                    };

                let main_window = match self.insert_window(window.0, window.1, None, style.clone())
                {
                    Ok(id) => id,
                    Err(e) => {
                        error!(error = ?e, "failed to insert main window runtime");
                        return;
                    }
                };
                self.app.with_global_mut(
                    fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
                    |svc, _app| {
                        svc.record_window_open(main_window, style, &caps);
                    },
                );
                self.main_window = Some(main_window);
                self.refresh_runner_monitor_topology_diagnostics(event_loop);
                #[cfg(feature = "dev-state")]
                self.dev_state.register_window_key(main_window, "main");
            }

            self.init_renderdoc_if_needed();

            if self.context.is_none() {
                let mut main_surface: Option<wgpu::Surface<'static>> = None;
                let context = match std::mem::replace(
                    &mut self.config.wgpu_init,
                    WgpuInit::CreateDefault,
                ) {
                    WgpuInit::CreateDefault => {
                        let context = match pollster::block_on(WgpuContext::new()) {
                            Ok(v) => v,
                            Err(e) => {
                                error!(error = ?e, "failed to initialize wgpu context");
                                return;
                            }
                        };

                        #[cfg(target_os = "android")]
                        {
                            let explicitly_requested_backend =
                                std::env::var_os("FRET_WGPU_BACKEND")
                                    .is_some_and(|v| !v.is_empty());
                            if !explicitly_requested_backend {
                                let info = context.adapter.get_info();
                                let name = info.name.to_ascii_lowercase();
                                let is_swiftshader = name.contains("swiftshader");
                                if is_swiftshader && info.backend == wgpu::Backend::Vulkan {
                                    error!(
                                        adapter = info.name,
                                        "wgpu Vulkan SwiftShader detected; Android emulator Vulkan is currently unstable (SIGSEGV in Renderer::new). Run on a real device or configure the emulator to use host Vulkan. Set FRET_WGPU_BACKEND to override."
                                    );
                                    self.wgpu_init_blocked = true;
                                    return;
                                }
                            }
                        }

                        context
                    }
                    WgpuInit::Provided(context) => context,
                    WgpuInit::Factory(factory) => {
                        let Some(main_window) = self.main_window else {
                            return;
                        };
                        let Some(window_ref) =
                            self.windows.get(main_window).map(|w| w.window.clone())
                        else {
                            return;
                        };

                        match factory(window_ref) {
                            Ok((context, surface)) => {
                                main_surface = Some(surface);
                                context
                            }
                            Err(e) => {
                                error!(error = ?e, "wgpu factory failed");
                                return;
                            }
                        }
                    }
                };

                self.publish_wgpu_adapter_selection_diagnostics(&context);

                let startup_async = self.install_renderer_bootstrap(context);

                if let Some(main_window) = self.main_window
                    && let Some(surface) = main_surface
                    && !self.attach_factory_surface_to_main_window(main_window, surface)
                {
                    return;
                }

                if let Some(main_window) = self.main_window {
                    if !self.driver_initialized {
                        self.driver.init(&mut self.app, main_window);
                        self.driver_initialized = true;
                        self.maybe_deliver_startup_incoming_open(main_window);
                        self.app.request_redraw(main_window);
                        if startup_async {
                            self.request_system_font_rescan();
                        }
                    }
                }
            }

            self.try_create_missing_surfaces();
            self.drain_effects(event_loop);
            return;
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let spec = self.config.main_window_spec();
            #[cfg(feature = "dev-state")]
            let spec = {
                let mut spec = spec;
                self.dev_state.apply_main_window_spec(&mut spec);
                self.dev_state.sanitize_window_spec_position(
                    "main",
                    &mut spec,
                    event_loop
                        .available_monitors()
                        .filter_map(|m| Some((m.position()?, m.current_video_mode()?.size()))),
                );
                spec
            };
            let style = self.config.main_window_style.clone();
            let caps = self
                .app
                .global::<fret_runtime::PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            let window = match self.create_os_window(event_loop, spec, style.clone(), None, &caps) {
                Ok(w) => w,
                Err(e) => {
                    error!(error = ?e, "failed to create main window");
                    return;
                }
            };

            // RenderDoc must be loaded/injected before the graphics API is initialized to reliably
            // hook Vulkan/D3D. Initialize capture integration before we create the wgpu context.
            self.init_renderdoc_if_needed();

            let (context, surface) =
                match std::mem::replace(&mut self.config.wgpu_init, WgpuInit::CreateDefault) {
                    WgpuInit::CreateDefault => {
                        match pollster::block_on(WgpuContext::new_with_surface(window.0.clone())) {
                            Ok(v) => v,
                            Err(e) => {
                                error!(error = ?e, "failed to initialize wgpu context");
                                return;
                            }
                        }
                    }
                    WgpuInit::Provided(context) => {
                        let surface = match context.create_surface(window.0.clone()) {
                            Ok(v) => v,
                            Err(e) => {
                                error!(
                                    error = ?e,
                                    "failed to create surface from provided context"
                                );
                                return;
                            }
                        };
                        (context, surface)
                    }
                    WgpuInit::Factory(factory) => match factory(window.0.clone()) {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = ?e, "wgpu factory failed");
                            return;
                        }
                    },
                };
            self.publish_wgpu_adapter_selection_diagnostics(&context);

            let startup_async = self.install_renderer_bootstrap(context);

            let main_window =
                match self.insert_window(window.0, window.1, Some(surface), style.clone()) {
                    Ok(id) => id,
                    Err(e) => {
                        error!(error = ?e, "failed to insert main window runtime");
                        return;
                    }
                };
            let caps = self
                .app
                .global::<fret_runtime::PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            self.app.with_global_mut(
                fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
                |svc, _app| {
                    svc.record_window_open(main_window, style, &caps);
                },
            );
            self.main_window = Some(main_window);
            self.refresh_runner_monitor_topology_diagnostics(event_loop);
            #[cfg(feature = "dev-state")]
            self.dev_state.register_window_key(main_window, "main");
            self.driver.init(&mut self.app, main_window);
            self.driver_initialized = true;
            self.maybe_deliver_startup_incoming_open(main_window);
            self.app.request_redraw(main_window);
            if startup_async {
                self.request_system_font_rescan();
            }
            self.drain_effects(event_loop);
        }
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.handle_destroy_surfaces();
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_proxy_wake_up(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app_window) = self.window_registry.get(window_id) else {
            return;
        };

        self.handle_window_pre_dispatch_event(app_window, &event);

        match event {
            #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
            WindowEvent::Moved(..) => {
                self.handle_window_moved();
            }
            ref ev @ WindowEvent::ModifiersChanged(..) => {
                self.handle_window_modifiers_changed(event_loop, app_window, ev);
            }
            WindowEvent::ThemeChanged(_theme) => {
                self.handle_window_theme_changed(app_window);
            }
            WindowEvent::Focused(focused) => {
                self.handle_window_focus_changed(app_window, window_id, focused);
            }
            WindowEvent::DragEntered { paths, position } => {
                self.handle_window_drag_entered(event_loop, app_window, paths, position);
            }
            WindowEvent::DragMoved { position } => {
                self.handle_window_drag_moved(event_loop, app_window, position);
            }
            WindowEvent::DragDropped { paths, position } => {
                self.handle_window_drag_dropped(event_loop, app_window, paths, position);
            }
            WindowEvent::DragLeft { position } => {
                self.handle_window_drag_left(event_loop, app_window, position);
            }
            WindowEvent::SurfaceResized(size) => {
                self.handle_window_surface_resized(event_loop, app_window, size);
            }
            ref ev @ WindowEvent::PointerMoved { .. } => {
                self.handle_window_pointer_moved(event_loop, app_window, ev);
            }
            ref ev @ WindowEvent::PointerButton { .. } => {
                self.handle_window_pointer_button(event_loop, app_window, ev);
            }
            WindowEvent::RedrawRequested => {
                self.handle_window_redraw_requested(event_loop, app_window);
            }
            ref ev => {
                self.handle_window_mapped_event(event_loop, app_window, ev);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.handle_about_to_wait_preamble(event_loop) {
            return;
        }

        #[cfg(feature = "diag-screenshots")]
        self.handle_about_to_wait_diag_screenshots();

        #[cfg(any(target_os = "android", target_os = "ios"))]
        self.handle_about_to_wait_mobile_surface_recreation(event_loop);

        self.handle_about_to_wait_internal_drag_poll(event_loop);
        let turn_now = self.handle_about_to_wait_turn_bookkeeping();

        #[cfg(all(
            feature = "dev-state",
            not(any(target_os = "android", target_os = "ios"))
        ))]
        self.handle_about_to_wait_dev_state_observation(turn_now);
        #[cfg(not(all(
            feature = "dev-state",
            not(any(target_os = "android", target_os = "ios"))
        )))]
        let _ = turn_now;

        self.handle_about_to_wait_window_platform_and_accessibility();

        self.handle_about_to_wait_dock_follow_stop();

        self.drain_effects(event_loop);

        let now = Instant::now();
        self.handle_about_to_wait_dock_released_outside_fallbacks(event_loop);

        self.handle_about_to_wait_control_flow(event_loop, now);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_resumed(event_loop);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_suspended(event_loop);
    }
}
