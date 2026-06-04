//! Winit `ApplicationHandler` integration.

use super::redraw_hitch::{
    RedrawPhase, measure_redraw_phase, quantize_logical_px, redraw_hitch_config,
    write_redraw_hitch_log,
};
use super::wheel_coalescing::{wheel_coalesce_delta, wheel_split_delta_by_max_abs_px};
use super::window::PendingWheelEvent;
use super::*;
use std::sync::Arc;

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

        if let Some(state) = self.windows.get_mut(app_window)
            && let Some(a11y) = state.accessibility.as_mut()
        {
            a11y.process_event(state.window.as_ref(), &event);
        }

        if let WindowEvent::Ime(ime) = &event
            && std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty())
            && let Some(state) = self.windows.get(app_window)
        {
            tracing::info!(
                "IME_DEBUG winit: WindowEvent::Ime({:?}) cached_rect={}",
                ime,
                state.platform.ime_cursor_area().is_some()
            );
        }

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

                if let Some(req) = self.poll_diag_wheel_burst_inject(app_window) {
                    let total_dx = req.delta_x * (req.count.max(1) as f32);
                    let total_dy = req.delta_y * (req.count.max(1) as f32);
                    let injected = PendingWheelEvent {
                        pointer_id: fret_core::PointerId(0),
                        position: req.position,
                        delta: fret_core::Point::new(
                            fret_core::Px(total_dx),
                            fret_core::Px(total_dy),
                        ),
                        modifiers: req.modifiers,
                        pointer_type: req.pointer_type,
                    };

                    if Self::wheel_coalescing_enabled() {
                        if let Some(state) = self.windows.get_mut(app_window) {
                            state.pending_wheel = Some(match state.pending_wheel.take() {
                                Some(mut prev) => {
                                    prev.delta = wheel_coalesce_delta(prev.delta, injected.delta);
                                    prev.position = injected.position;
                                    prev.modifiers = injected.modifiers;
                                    prev.pointer_type = injected.pointer_type;
                                    prev.pointer_id = injected.pointer_id;
                                    prev
                                }
                                None => injected,
                            });
                        }
                        self.app.request_redraw(app_window);
                    } else {
                        self.deliver_window_event_now(
                            app_window,
                            &Event::Pointer(fret_core::PointerEvent::Wheel {
                                pointer_id: injected.pointer_id,
                                position: injected.position,
                                delta: injected.delta,
                                modifiers: injected.modifiers,
                                pointer_type: injected.pointer_type,
                            }),
                        );
                    }
                }

                if Self::wheel_coalescing_enabled() {
                    let max_abs = Self::wheel_coalescing_max_abs_px();
                    let mut to_deliver: Option<PendingWheelEvent> = None;
                    let mut remainder: Option<PendingWheelEvent> = None;

                    if let Some(state) = self.windows.get_mut(app_window)
                        && let Some(pending) = state.pending_wheel.take()
                    {
                        let (delivered_delta, remainder_delta) =
                            wheel_split_delta_by_max_abs_px(pending.delta, max_abs);
                        if delivered_delta.x.0.abs() > 0.0001 || delivered_delta.y.0.abs() > 0.0001
                        {
                            to_deliver = Some(PendingWheelEvent {
                                delta: delivered_delta,
                                ..pending
                            });
                        }
                        if remainder_delta.x.0.abs() > 0.0001 || remainder_delta.y.0.abs() > 0.0001
                        {
                            remainder = Some(PendingWheelEvent {
                                delta: remainder_delta,
                                ..pending
                            });
                        }
                    }

                    if let Some(remainder) = remainder {
                        if let Some(state) = self.windows.get_mut(app_window) {
                            state.pending_wheel = Some(remainder);
                        }
                        self.app.request_redraw(app_window);
                    }

                    if let Some(wheel) = to_deliver {
                        self.deliver_window_event_now(
                            app_window,
                            &Event::Pointer(fret_core::PointerEvent::Wheel {
                                pointer_id: wheel.pointer_id,
                                position: wheel.position,
                                delta: wheel.delta,
                                modifiers: wheel.modifiers,
                                pointer_type: wheel.pointer_type,
                            }),
                        );
                    }
                }

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
                if let Some(diag) = self.diag_screenshots.as_mut() {
                    diag.poll();
                }

                if let Some(size) = self
                    .windows
                    .get_mut(app_window)
                    .and_then(|state| state.pending_surface_resize.take())
                {
                    // The platform event path now reconfigures the surface immediately. Keep the
                    // redraw-time resize as an eventual-consistency fallback for windows that
                    // queued a size before their surface/context existed.
                    self.resize_surface(app_window, size.width, size.height);

                    // Keep delivering size/scale events once per frame so interactive resizes do
                    // not spam high-level relayout work even though the GPU surface has already
                    // been synchronized to the latest physical size.
                    let (logical_width, logical_height, scale_factor, should_deliver_resized) = {
                        let Some(state) = self.windows.get_mut(app_window) else {
                            return;
                        };
                        let scale_factor = state.window.scale_factor() as f32;
                        let logical: winit::dpi::LogicalSize<f32> =
                            size.to_logical(state.window.scale_factor());
                        let logical_width = quantize_logical_px(logical.width);
                        let logical_height = quantize_logical_px(logical.height);
                        let bits = (logical_width.to_bits(), logical_height.to_bits());
                        let should_deliver_resized = state
                            .last_delivered_window_resized
                            .is_none_or(|prev| prev != bits);
                        if should_deliver_resized {
                            state.last_delivered_window_resized = Some(bits);
                        }
                        (
                            logical_width,
                            logical_height,
                            scale_factor,
                            should_deliver_resized,
                        )
                    };

                    if should_deliver_resized {
                        self.deliver_window_event_now(
                            app_window,
                            &Event::WindowResized {
                                width: Px(logical_width),
                                height: Px(logical_height),
                            },
                        );
                    }
                    let should_deliver_scale_factor = self
                        .app
                        .global::<fret_core::WindowMetricsService>()
                        .and_then(|svc| svc.scale_factor(app_window))
                        .is_none_or(|prev| prev.to_bits() != scale_factor.to_bits());
                    if should_deliver_scale_factor {
                        self.deliver_window_event_now(
                            app_window,
                            &Event::WindowScaleFactorChanged(scale_factor),
                        );
                    }
                }

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

                    let ((scale_factor, bounds), prepare_elapsed) =
                        measure_redraw_phase(RedrawPhase::Prepare, hitch_config.is_some(), || {
                            // Apply any pending window-side state (IME/cursor) once per frame,
                            // similar to Dear ImGui's backend `prepare_frame` pattern.
                            state.platform.prepare_frame(state.window.as_ref());

                            let scale_factor = state.window.scale_factor() as f32;
                            let physical = state.window.surface_size();
                            let logical: winit::dpi::LogicalSize<f32> =
                                physical.to_logical(state.window.scale_factor());
                            let logical_width = quantize_logical_px(logical.width);
                            let logical_height = quantize_logical_px(logical.height);

                            let bounds = Rect::new(
                                Point::new(Px(0.0), Px(0.0)),
                                Size::new(Px(logical_width), Px(logical_height)),
                            );

                            self.driver.gpu_frame_prepare(
                                &mut self.app,
                                app_window,
                                &mut state.user,
                                context,
                                renderer,
                                scale_factor,
                            );

                            (scale_factor, bounds)
                        });
                    if let Some(elapsed) = prepare_elapsed {
                        hitch_prepare_ms = Some(elapsed.as_millis() as u64);
                    }

                    let render_text_debug_enabled =
                        std::env::var_os("FRET_RENDER_TEXT_DEBUG").is_some_and(|v| !v.is_empty());
                    let render_text_diag_enabled = std::env::var_os("FRET_DIAG_DIR")
                        .is_some_and(|v| !v.is_empty())
                        || render_text_debug_enabled;
                    let (_, render_elapsed) = measure_redraw_phase(
                        RedrawPhase::Render {
                            bounds,
                            scale_factor,
                        },
                        hitch_config.is_some(),
                        || {
                            if render_text_diag_enabled {
                                renderer.begin_text_diagnostics_frame();
                            }
                            self.driver.render(WinitRenderContext {
                                app: &mut self.app,
                                services: renderer as &mut dyn fret_core::UiServices,
                                window: app_window,
                                state: &mut state.user,
                                bounds,
                                scale_factor,
                                scene: &mut state.scene,
                            });
                        },
                    );
                    if let Some(elapsed) = render_elapsed {
                        hitch_render_ms = Some(elapsed.as_millis() as u64);
                    }

                    // Consume the window-scoped text-input snapshot after render so the runner can
                    // position the IME candidate window based on the final painted caret rect.
                    //
                    // Note: v1 still emits `Effect::ImeSetCursorArea` from widgets; this snapshot
                    // path is a runner-level fallback and an integration seam for future macOS
                    // (NSTextInputClient) interop.
                    if let Some(snapshot) = self
                        .app
                        .global::<fret_runtime::WindowTextInputSnapshotService>()
                        .and_then(|svc| svc.snapshot(app_window))
                    {
                        let mut dirty = false;
                        let ime_changed =
                            state.platform.set_ime_allowed(snapshot.focus_is_text_input);
                        dirty |= ime_changed;
                        #[cfg(target_os = "android")]
                        if ime_changed {
                            android_soft_input_request = Some(snapshot.focus_is_text_input);
                        }
                        if snapshot.focus_is_text_input
                            && let Some(rect) = snapshot.ime_cursor_area
                        {
                            dirty |= state.platform.set_ime_cursor_area(rect);
                        }
                        if snapshot.focus_is_text_input {
                            let surrounding = snapshot.surrounding_text.as_ref().map(|s| {
                                fret_runner_winit::ImeSurroundingTextUpdate {
                                    text: Arc::clone(&s.text),
                                    cursor: s.cursor,
                                    anchor: s.anchor,
                                }
                            });
                            dirty |= state.platform.set_ime_surrounding_text(surrounding);
                        } else {
                            dirty |= state.platform.set_ime_surrounding_text(None);
                        }

                        if dirty {
                            if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
                                tracing::info!(
                                    "IME_DEBUG snapshot: window={:?} focus={} cursor_area={:?}",
                                    app_window,
                                    snapshot.focus_is_text_input,
                                    snapshot.ime_cursor_area
                                );
                            }
                            state.platform.prepare_frame(state.window.as_ref());
                        }
                    }

                    super::render::validate_scene_if_enabled(&state.scene);

                    if let Some(a11y) = state.accessibility.as_mut()
                        && a11y.is_active()
                        && let Some(snapshot) = self.driver.semantics_snapshot(
                            &mut self.app,
                            app_window,
                            &mut state.user,
                        )
                    {
                        let update = accessibility::tree_update_from_snapshot(
                            &snapshot,
                            state.window.scale_factor(),
                        );
                        a11y.update_if_active(|| update);
                        state.last_semantics_snapshot = Some(snapshot);
                    } else {
                        state.last_semantics_snapshot = None;
                    }

                    let (engine_frame, record_elapsed) = measure_redraw_phase(
                        RedrawPhase::Record {
                            scene_ops: state.scene.ops_len(),
                        },
                        hitch_config.is_some(),
                        || {
                            self.driver.record_engine_frame(
                                &mut self.app,
                                app_window,
                                &mut state.user,
                                context,
                                renderer,
                                scale_factor,
                                self.tick_id,
                                self.frame_id,
                            )
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

                    for update in target_updates {
                        match update {
                            RenderTargetUpdate::Update { id, desc } => {
                                if !renderer.update_render_target(id, desc) {
                                    error!(
                                        ?id,
                                        "engine frame update tried to update unknown render target"
                                    );
                                }
                            }
                            RenderTargetUpdate::Unregister { id } => {
                                if !renderer.unregister_render_target(id) {
                                    error!(
                                        ?id,
                                        "engine frame update tried to unregister unknown render target"
                                    );
                                }
                            }
                        }
                    }

                    let (draw_result, present_elapsed) = measure_redraw_phase(
                        RedrawPhase::Present,
                        hitch_config.is_some(),
                        || -> Result<(), fret_render::RenderError> {
                            let frame_view = match surface.get_current_frame_view() {
                                Ok(frame_view) => Some(frame_view),
                                Err(source) => {
                                    let diag_renderer_perf =
                                        std::env::var_os("FRET_DIAG_RENDERER_PERF")
                                            .is_some_and(|v| !v.is_empty());
                                    if !diag_renderer_perf
                                        || source != fret_render::SurfaceAcquireError::Other
                                    {
                                        return Err(
                                            fret_render::RenderError::SurfaceAcquireFailed {
                                                source,
                                            },
                                        );
                                    }
                                    None
                                }
                            };

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
                            let fallback_target = if frame_view.is_none() {
                                Some(context.device.create_texture(&wgpu::TextureDescriptor {
                                    label: Some("fret diag renderer perf fallback target"),
                                    size: wgpu::Extent3d {
                                        width: surface.size().0.max(1),
                                        height: surface.size().1.max(1),
                                        depth_or_array_layers: 1,
                                    },
                                    mip_level_count: 1,
                                    sample_count: 1,
                                    dimension: wgpu::TextureDimension::D2,
                                    format: surface.format(),
                                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                    view_formats: &[],
                                }))
                            } else {
                                None
                            };
                            let fallback_view = fallback_target.as_ref().map(|target| {
                                target.create_view(&wgpu::TextureViewDescriptor::default())
                            });
                            let target_view = frame_view
                                .as_ref()
                                .map(|(_, view)| view)
                                .or(fallback_view.as_ref())
                                .expect("renderer perf fallback should provide a target view");

                            let (ui_cmd, _) =
                                measure_redraw_phase(RedrawPhase::RenderScene, false, || {
                                    renderer.render_scene(
                                        &context.device,
                                        &context.queue,
                                        fret_render::RenderSceneParams {
                                            format: surface.format(),
                                            target_view,
                                            scene: &state.scene,
                                            clear: clear_color,
                                            scale_factor,
                                            viewport_size: surface.size(),
                                        },
                                    )
                                });
                            crate::runner::font_catalog::publish_renderer_svg_text_bridge_diagnostics(
                            &mut self.app,
                            renderer,
                        );
                            if render_text_diag_enabled {
                                let diagnostics = renderer.text_diagnostics_snapshot(self.frame_id);
                                let trace = renderer.text_font_trace_snapshot(self.frame_id);
                                let policy = renderer.text_fallback_policy_snapshot(self.frame_id);

                                if render_text_debug_enabled {
                                    self.app.set_global(diagnostics);
                                    self.app.set_global(trace);
                                    self.app.set_global(policy);
                                } else {
                                    // Avoid turning per-frame diagnostics snapshots into global-change
                                    // propagation / invalidation work during perf-sensitive runs.
                                    self.app.with_global_mut_untracked(
                                        fret_core::RendererTextPerfSnapshot::default,
                                        |slot, _app| {
                                            *slot = diagnostics;
                                        },
                                    );
                                    self.app.with_global_mut_untracked(
                                        fret_core::RendererTextFontTraceSnapshot::default,
                                        |slot, _app| {
                                            *slot = trace;
                                        },
                                    );
                                    self.app.with_global_mut_untracked(
                                        fret_core::RendererTextFallbackPolicySnapshot::default,
                                        |slot, _app| {
                                            *slot = policy;
                                        },
                                    );
                                }
                            }

                            let diag_renderer_perf = std::env::var_os("FRET_DIAG_RENDERER_PERF")
                                .is_some_and(|v| !v.is_empty());
                            if diag_renderer_perf {
                                let tick_id = self.tick_id.0;
                                let frame_id = self.frame_id.0;
                                let sample = renderer.take_last_frame_perf_snapshot().map(|perf| {
                                    fret_render::RendererPerfFrameSample {
                                        tick_id,
                                        frame_id,
                                        perf,
                                    }
                                });
                                if let Some(sample) = sample {
                                    self.app.with_global_mut_untracked(
                                        fret_render::RendererPerfFrameStore::default,
                                        |store, _app| {
                                            store.record(
                                                app_window,
                                                tick_id,
                                                frame_id,
                                                sample.perf,
                                            );
                                        },
                                    );
                                }
                                self.driver.renderer_perf_sample(
                                    &mut self.app,
                                    app_window,
                                    &mut state.user,
                                    sample,
                                );
                            }

                            let diag_wgpu_report = std::env::var_os("FRET_DIAG_WGPU_REPORT")
                                .is_some_and(|v| !v.is_empty());
                            if diag_wgpu_report {
                                let every_n = std::env::var("FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES")
                                    .ok()
                                    .and_then(|v| v.trim().parse::<u64>().ok())
                                    .unwrap_or(60)
                                    .max(1);

                                let tick_id = self.tick_id.0;
                                let frame_id = self.frame_id.0;
                                let should_sample =
                                    frame_id <= 2 || frame_id.is_multiple_of(every_n);

                                if should_sample
                                    && let Some(report) = context.instance.generate_report()
                                {
                                    let hub = report.hub_report();
                                    let counts = fret_render::WgpuHubReportCounts {
                                        adapters: (hub.adapters.num_allocated
                                            + hub.adapters.num_kept_from_user)
                                            as u64,
                                        devices: (hub.devices.num_allocated
                                            + hub.devices.num_kept_from_user)
                                            as u64,
                                        queues: (hub.queues.num_allocated
                                            + hub.queues.num_kept_from_user)
                                            as u64,
                                        command_encoders: (hub.command_encoders.num_allocated
                                            + hub.command_encoders.num_kept_from_user)
                                            as u64,
                                        buffers: (hub.buffers.num_allocated
                                            + hub.buffers.num_kept_from_user)
                                            as u64,
                                        textures: (hub.textures.num_allocated
                                            + hub.textures.num_kept_from_user)
                                            as u64,
                                        texture_views: (hub.texture_views.num_allocated
                                            + hub.texture_views.num_kept_from_user)
                                            as u64,
                                        samplers: (hub.samplers.num_allocated
                                            + hub.samplers.num_kept_from_user)
                                            as u64,
                                        shader_modules: (hub.shader_modules.num_allocated
                                            + hub.shader_modules.num_kept_from_user)
                                            as u64,
                                        render_pipelines: (hub.render_pipelines.num_allocated
                                            + hub.render_pipelines.num_kept_from_user)
                                            as u64,
                                        compute_pipelines: (hub.compute_pipelines.num_allocated
                                            + hub.compute_pipelines.num_kept_from_user)
                                            as u64,
                                    };

                                    self.app.with_global_mut_untracked(
                                        fret_render::WgpuHubReportFrameStore::default,
                                        |store, _app| {
                                            store.record(app_window, tick_id, frame_id, counts);
                                        },
                                    );
                                }
                            }

                            let diag_wgpu_allocator_report =
                                std::env::var_os("FRET_DIAG_WGPU_ALLOCATOR_REPORT")
                                    .is_some_and(|v| !v.is_empty());
                            if diag_wgpu_allocator_report {
                                let every_n =
                                    std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES")
                                        .ok()
                                        .and_then(|v| v.trim().parse::<u64>().ok())
                                        .unwrap_or(300)
                                        .max(1);
                                let top_n = std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_TOP_N")
                                    .ok()
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                                    .unwrap_or(16)
                                    .max(1);
                                let max_name_bytes =
                                    std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_MAX_NAME_BYTES")
                                        .ok()
                                        .and_then(|v| v.trim().parse::<usize>().ok())
                                        .unwrap_or(160)
                                        .max(16);

                                let tick_id = self.tick_id.0;
                                let frame_id = self.frame_id.0;
                                let should_sample =
                                    frame_id <= 2 || frame_id.is_multiple_of(every_n);

                                if should_sample {
                                    let report = context.device.generate_allocator_report();
                                    #[cfg(target_os = "macos")]
                                    let metal_current_allocated_size_bytes = unsafe {
                                        context.device.as_hal::<wgpu::hal::api::Metal>().map(
                                            |dev| dev.raw_device().currentAllocatedSize() as u64,
                                        )
                                    };
                                    #[cfg(not(target_os = "macos"))]
                                let metal_current_allocated_size_bytes: Option<u64> = None;

                                    self.app.with_global_mut_untracked(
                                        fret_render::WgpuAllocatorReportFrameStore::default,
                                        |store, _app| {
                                            store.record_sample(
                                                app_window,
                                                tick_id,
                                                frame_id,
                                                report,
                                                metal_current_allocated_size_bytes,
                                                top_n,
                                                max_name_bytes,
                                            );
                                        },
                                    );
                                }
                            }

                            let mut cmd_buffers = engine_command_buffers;
                            cmd_buffers.push(ui_cmd);

                            #[cfg(feature = "diag-screenshots")]
                            let mut screenshot_inflight: Option<
                                diag_screenshots::InFlightCapture,
                            > = None;
                            #[cfg(feature = "diag-screenshots")]
                            if let (Some(diag), Some((frame, _view))) =
                                (self.diag_screenshots.as_mut(), frame_view.as_ref())
                            {
                                let window_ffi = app_window.data().as_ffi();
                                if let Some((cmd, inflight)) = diag.begin_capture_for_window(
                                    &context.device,
                                    window_ffi,
                                    &frame.texture,
                                    surface.format(),
                                    surface.size(),
                                ) {
                                    cmd_buffers.push(cmd);
                                    screenshot_inflight = Some(inflight);
                                }
                            }

                            let mut pending_bundle_screenshot = None;
                            if let (Some(dir), Some((frame, _view))) =
                                (screenshot_dir, frame_view.as_ref())
                                && let Some((pending, copy_cmd)) =
                                    self.diag_bundle_screenshots.begin_readback(
                                        &context.device,
                                        &frame.texture,
                                        surface.format(),
                                        surface.size(),
                                    )
                            {
                                cmd_buffers.push(copy_cmd);
                                pending_bundle_screenshot = Some((pending, dir));
                            }

                            context.queue.submit(cmd_buffers);
                            if let Some((frame, _view)) = frame_view {
                                frame.present();
                            }
                            super::scheduling_diagnostics::commit_presented_frame_for_window(
                                &mut self.app,
                                &mut self.frame_id,
                                app_window,
                            );
                            drop(engine_keepalive);

                            #[cfg(feature = "diag-screenshots")]
                            if let (Some(diag), Some(inflight)) =
                                (self.diag_screenshots.as_mut(), screenshot_inflight)
                                && let Err(err) = diag.finish_capture(&context.device, inflight)
                            {
                                tracing::warn!(
                                    error = %err,
                                    window = ?app_window,
                                    "diag screenshot: capture failed"
                                );
                            }

                            if let Some((pending, dir)) = pending_bundle_screenshot {
                                let _ = self.diag_bundle_screenshots.finish_and_write_bmp(
                                    &context.device,
                                    pending,
                                    &dir,
                                    surface.format(),
                                );
                            }

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
