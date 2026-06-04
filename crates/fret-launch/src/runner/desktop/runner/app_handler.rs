//! Winit `ApplicationHandler` integration.

use super::redraw_hitch::{
    RedrawPhase, measure_redraw_phase, redraw_hitch_config, write_redraw_hitch_log,
};
use super::*;

use fret_core::time::Instant;
#[cfg(target_os = "macos")]
use objc2_metal::MTLDevice as _;

#[cfg(feature = "diag-screenshots")]
use slotmap::Key as _;

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
                let redraw_span = tracing::info_span!(
                    "fret.runner.redraw",
                    window = ?app_window,
                    tick_id = self.app.tick_id().0,
                    frame_id = self.app.frame_id().0,
                );
                let _redraw_guard = redraw_span.enter();

                self.handle_window_redraw_pending_wheel(app_window);

                let window_ref = self.windows.get(app_window).map(|s| s.window.clone());
                if let Some(window_ref) = window_ref {
                    let _ = self
                        .update_window_environment_for_window_ref(app_window, window_ref.as_ref());
                }

                let hitch_config = redraw_hitch_config();
                let hitch_total_started = hitch_config.map(|_| Instant::now());
                let mut hitch_prepare_ms: Option<u64> = None;
                let mut hitch_render_ms: Option<u64> = None;
                let mut hitch_record_ms: Option<u64> = None;
                let mut hitch_present_ms: Option<u64> = None;

                // Drain effects before rendering so dock ops, invalidation bumps, and window
                // requests apply deterministically to the frame being drawn (ADR 0013).
                self.drain_effects(event_loop);

                #[cfg(feature = "diag-screenshots")]
                super::window_redraw_diag_screenshots::poll_window_redraw_diag_screenshot_requests(
                    self.diag_screenshots.as_mut(),
                );

                self.handle_window_redraw_pending_surface_resize(app_window);

                #[cfg(target_os = "android")]
                let mut android_soft_input_request: Option<bool> = None;

                {
                    let (Some(context), Some(renderer)) =
                        (self.context.as_ref(), self.renderer.as_mut())
                    else {
                        return;
                    };
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let Some(surface) = state.surface.as_mut() else {
                        return;
                    };

                    let capturing = self
                        .renderdoc
                        .as_mut()
                        .is_some_and(|r| r.begin_capture_if_requested());

                    let (prepared, prepare_elapsed) =
                        super::window_redraw_frame_prepare::prepare_window_redraw_frame(
                            super::window_redraw_frame_prepare::WindowRedrawFramePrepareInput {
                                app: &mut self.app,
                                driver: &mut self.driver,
                                app_window,
                                user: &mut state.user,
                                platform: &mut state.platform,
                                window: state.window.as_ref(),
                                context,
                                renderer,
                                hitch_enabled: hitch_config.is_some(),
                            },
                        );
                    let scale_factor = prepared.scale_factor;
                    let bounds = prepared.bounds;
                    if let Some(elapsed) = prepare_elapsed {
                        hitch_prepare_ms = Some(elapsed.as_millis() as u64);
                    }

                    let (rendered, render_elapsed) =
                        super::window_redraw_render::render_window_redraw_frame(
                            super::window_redraw_render::WindowRedrawRenderInput {
                                app: &mut self.app,
                                driver: &mut self.driver,
                                renderer,
                                app_window,
                                user: &mut state.user,
                                scene: &mut state.scene,
                                bounds,
                                scale_factor,
                                hitch_enabled: hitch_config.is_some(),
                            },
                        );
                    let text_diagnostics = rendered.text_diagnostics;
                    if let Some(elapsed) = render_elapsed {
                        hitch_render_ms = Some(elapsed.as_millis() as u64);
                    }

                    #[cfg(target_os = "android")]
                    super::window_redraw_text_input::apply_window_redraw_text_input_snapshot(
                        &self.app,
                        app_window,
                        &mut state.platform,
                        state.window.as_ref(),
                        &mut android_soft_input_request,
                    );
                    #[cfg(not(target_os = "android"))]
                    super::window_redraw_text_input::apply_window_redraw_text_input_snapshot(
                        &self.app,
                        app_window,
                        &mut state.platform,
                        state.window.as_ref(),
                    );

                    super::render::validate_scene_if_enabled(&state.scene);

                    let accessibility_scale_factor = state.window.scale_factor();
                    super::window_redraw_accessibility::update_window_redraw_accessibility_snapshot(
                        &mut self.driver,
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        &mut state.accessibility,
                        &mut state.last_semantics_snapshot,
                        accessibility_scale_factor,
                    );

                    let (engine_frame, record_elapsed) =
                        super::window_redraw_record::record_window_redraw_frame(
                            super::window_redraw_record::WindowRedrawRecordInput {
                                app: &mut self.app,
                                driver: &mut self.driver,
                                app_window,
                                user: &mut state.user,
                                context,
                                renderer,
                                scale_factor,
                                tick_id: self.tick_id,
                                frame_id: self.frame_id,
                                scene_ops: state.scene.ops_len(),
                                hitch_enabled: hitch_config.is_some(),
                            },
                        );
                    let EngineFrameUpdate {
                        target_updates,
                        command_buffers: engine_command_buffers,
                        keepalive: engine_keepalive,
                    } = engine_frame;
                    if let Some(elapsed) = record_elapsed {
                        hitch_record_ms = Some(elapsed.as_millis() as u64);
                    }

                    #[cfg(feature = "webview-wry")]
                    let webview_snapshot =
                        if self.app.global::<fret_webview::WebViewHost>().is_some()
                            && fret_webview::webview_has_surfaces_for_window(&self.app, app_window)
                        {
                            state.last_semantics_snapshot.clone().or_else(|| {
                                self.driver.semantics_snapshot(
                                    &mut self.app,
                                    app_window,
                                    &mut state.user,
                                )
                            })
                        } else {
                            None
                        };
                    #[cfg(not(feature = "webview-wry"))]
                    let webview_snapshot: Option<
                        std::sync::Arc<fret_core::SemanticsSnapshot>,
                    > = None;

                    self.webviews.sync_window(
                        &mut self.app,
                        self.frame_id,
                        app_window,
                        state.window.as_ref(),
                        webview_snapshot.as_ref(),
                    );

                    super::window_redraw_target_updates::apply_window_redraw_target_updates(
                        renderer,
                        target_updates,
                    );

                    let (draw_result, present_elapsed) = measure_redraw_phase(
                        RedrawPhase::Present,
                        hitch_config.is_some(),
                        || -> Result<(), fret_render::RenderError> {
                            let frame_view =
                                super::window_redraw_present_target::acquire_window_redraw_present_frame(
                                    surface,
                                )?;

                            let screenshot_dir = self.diag_bundle_screenshots.poll_request_dir();

                            let want_visual_transparent = self
                                .app
                                .global::<fret_runtime::RunnerWindowStyleDiagnosticsStore>()
                                .and_then(|s| s.effective_snapshot(app_window))
                                .is_some_and(|s| s.visual_transparent);
                            let clear_color = if want_visual_transparent {
                                fret_render::ClearColor(wgpu::Color::TRANSPARENT)
                            } else {
                                self.config.clear_color
                            };
                            let present_target =
                                super::window_redraw_present_target::prepare_window_redraw_present_target(
                                    super::window_redraw_present_target::WindowRedrawPresentTargetInput {
                                        context,
                                        surface,
                                        frame_view,
                                    },
                                );

                            let (ui_cmd, _) =
                                measure_redraw_phase(RedrawPhase::RenderScene, false, || {
                                    renderer.render_scene(
                                        &context.device,
                                        &context.queue,
                                        fret_render::RenderSceneParams {
                                            format: surface.format(),
                                            target_view: present_target.target_view(),
                                            scene: &state.scene,
                                            clear: clear_color,
                                            scale_factor,
                                            viewport_size: surface.size(),
                                        },
                                    )
                                });
                            super::window_redraw_text_diagnostics::publish_window_redraw_text_diagnostics(
                                &mut self.app,
                                renderer,
                                self.frame_id,
                                text_diagnostics,
                            );

                            super::window_redraw_renderer_perf::maybe_publish_window_redraw_renderer_perf_sample(
                                &mut self.app,
                                &mut self.driver,
                                renderer,
                                app_window,
                                &mut state.user,
                                self.tick_id.0,
                                self.frame_id.0,
                            );

                            super::window_redraw_wgpu_report::maybe_record_window_redraw_wgpu_hub_report(
                                &mut self.app,
                                context,
                                app_window,
                                self.tick_id.0,
                                self.frame_id.0,
                            );

                            super::window_redraw_wgpu_allocator_report::maybe_record_window_redraw_wgpu_allocator_report(
                                &mut self.app,
                                context,
                                app_window,
                                self.tick_id.0,
                                self.frame_id.0,
                            );

                            let mut cmd_buffers = engine_command_buffers;
                            cmd_buffers.push(ui_cmd);

                            #[cfg(feature = "diag-screenshots")]
                            let screenshot_inflight =
                                super::window_redraw_diag_screenshots::begin_window_redraw_diag_screenshot_capture(
                                     self.diag_screenshots.as_mut(),
                                     app_window,
                                     present_target.frame_view(),
                                     &context.device,
                                     surface.format(),
                                     surface.size(),
                                    &mut cmd_buffers,
                                );
                            let pending_bundle_screenshot =
                                super::window_redraw_diag_screenshots::begin_window_redraw_bundle_screenshot_readback(
                                     &self.diag_bundle_screenshots,
                                     screenshot_dir,
                                     present_target.frame_view(),
                                     &context.device,
                                     surface.format(),
                                     surface.size(),
                                    &mut cmd_buffers,
                                );

                            context.queue.submit(cmd_buffers);
                            if let Some((frame, _view)) = present_target.into_frame_view() {
                                frame.present();
                            }
                            super::scheduling_diagnostics::commit_presented_frame_for_window(
                                &mut self.app,
                                &mut self.frame_id,
                                app_window,
                            );
                            drop(engine_keepalive);

                            #[cfg(feature = "diag-screenshots")]
                            super::window_redraw_diag_screenshots::finish_window_redraw_diag_screenshot_capture(
                                self.diag_screenshots.as_mut(),
                                &context.device,
                                app_window,
                                screenshot_inflight,
                            );

                            super::window_redraw_diag_screenshots::finish_window_redraw_bundle_screenshot_readback(
                                &self.diag_bundle_screenshots,
                                &context.device,
                                pending_bundle_screenshot,
                                surface.format(),
                            );

                            Ok(())
                        },
                    );
                    if let Some(elapsed) = present_elapsed {
                        hitch_present_ms = Some(elapsed.as_millis() as u64);
                    }

                    if capturing && let Some(r) = self.renderdoc.as_mut() {
                        r.end_capture();
                    }

                    if let Err(err) = draw_result {
                        match err {
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: fret_render::SurfaceAcquireError::Lost,
                            } => {
                                let _ = surface;
                                state.surface = None;
                                let _ = state;
                                let _ = self.request_window_redraw_with_reason(
                                    app_window,
                                    fret_runtime::RunnerFrameDriveReason::SurfaceRecoverLost,
                                );
                                self.raf_windows.request(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: fret_render::SurfaceAcquireError::Outdated,
                            } => {
                                let _ = surface;
                                state.surface = None;
                                let _ = state;
                                let _ = self.request_window_redraw_with_reason(
                                    app_window,
                                    fret_runtime::RunnerFrameDriveReason::SurfaceRecoverOutdated,
                                );
                                self.raf_windows.request(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: fret_render::SurfaceAcquireError::Timeout,
                            } => {
                                // Transient on some platforms (especially during startup / resize).
                                // Schedule a one-shot redraw so the window doesn't stay blank until
                                // the next user input arrives.
                                let _ = state;
                                let _ = self.request_window_redraw_with_reason(
                                    app_window,
                                    fret_runtime::RunnerFrameDriveReason::SurfaceRecoverTimeout,
                                );
                                self.raf_windows.request(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: fret_render::SurfaceAcquireError::OutOfMemory,
                            } => {
                                self.dispatcher.shutdown();
                                event_loop.exit();
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed { .. } => return,
                            _ => {
                                error!(?err, "render error");
                                return;
                            }
                        }
                    }

                    if let (Some(cfg), Some(started)) = (hitch_config, hitch_total_started) {
                        let total_ms = started.elapsed().as_millis() as u64;
                        if total_ms >= cfg.hitch_ms {
                            write_redraw_hitch_log(&format!(
                                "redraw hitch window={app_window:?} tick_id={tick_id} frame_id={frame_id} total_ms={total_ms} prepare_ms={prepare_ms:?} render_ms={render_ms:?} record_ms={record_ms:?} present_ms={present_ms:?} scene_ops={scene_ops} bounds={bounds:?} scale_factor={scale_factor}",
                                tick_id = self.tick_id.0,
                                frame_id = self.frame_id.0,
                                prepare_ms = hitch_prepare_ms,
                                render_ms = hitch_render_ms,
                                record_ms = hitch_record_ms,
                                present_ms = hitch_present_ms,
                                scene_ops = state.scene.ops_len(),
                            ));
                        }
                    }
                }

                #[cfg(target_os = "android")]
                if let Some(enabled) = android_soft_input_request {
                    self.android_force_soft_input(enabled);
                }

                // Drain effects produced during rendering so they don't lag by a frame (e.g. IME
                // cursor updates, timer-driven docking invalidations, window raise/create effects).
                self.drain_effects(event_loop);
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
