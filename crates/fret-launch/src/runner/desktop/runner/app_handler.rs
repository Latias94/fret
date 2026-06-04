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
use fret_platform::external_drop::ExternalDropProvider as _;
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
                if macos_hit_test::has_active_regions() {
                    macos_hit_test::apply_latest_mouse_location();
                }
            }
            ref ev @ WindowEvent::ModifiersChanged(..) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.platform.handle_window_event(
                        state.window.scale_factor(),
                        ev,
                        &mut Vec::new(),
                    );
                }

                if self.internal_drag_routing_pointer_id().is_some() {
                    self.route_internal_drag_hover_from_cursor();
                    self.drain_effects(event_loop);
                }
            }
            WindowEvent::ThemeChanged(_theme) => {
                let window_ref = self.windows.get(app_window).map(|s| s.window.clone());
                if let Some(window_ref) = window_ref
                    && self
                        .update_window_environment_for_window_ref(app_window, window_ref.as_ref())
                {
                    self.app.request_redraw(app_window);
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.is_focused = focused;
                    if !focused {
                        state.platform.input.pressed_buttons = fret_core::MouseButtons::default();
                    }
                }
                if focused {
                    self.bump_window_z_order(app_window);
                }
                self.deliver_window_event_now(app_window, &Event::WindowFocusChanged(focused));
                macos_window_log(format_args!(
                    "[focused] app_window={:?} focused={} winit={:?}",
                    app_window, focused, window_id
                ));
            }
            WindowEvent::DragEntered { paths, position } => {
                tracing::debug!(count = paths.len(), "winit drag entered");
                let existing = self
                    .windows
                    .get(app_window)
                    .and_then(|s| s.external_drag_token);
                let token = existing.unwrap_or_else(|| self.external_drop.allocate_token());

                let (position, kind, files) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    if state.external_drag_token.is_none() {
                        state.external_drag_token = Some(token);
                    }
                    let position = fret_runner_winit::map_physical_position_to_point(
                        state.window.scale_factor(),
                        position,
                    );
                    state.external_drag_files = paths;
                    let files = state.external_drag_files.clone();
                    let kind = ExternalDragKind::EnterFiles(
                        fret_runner_winit::external_drag_files(token, &files),
                    );
                    (position, kind, files)
                };

                self.external_drop.set_payload_paths(token, files);

                self.deliver_window_event_now(
                    app_window,
                    &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                );
                self.drain_effects(event_loop);
            }
            WindowEvent::DragMoved { position } => {
                let (position, token) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    let position = fret_runner_winit::map_physical_position_to_point(
                        state.window.scale_factor(),
                        position,
                    );
                    (position, state.external_drag_token)
                };

                if let Some(token) = token {
                    let paths = self.external_drop.paths(token).unwrap_or(&[]);
                    let kind = ExternalDragKind::OverFiles(fret_runner_winit::external_drag_files(
                        token, paths,
                    ));
                    self.deliver_window_event_now(
                        app_window,
                        &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::DragDropped { paths, position } => {
                tracing::debug!(count = paths.len(), "winit drag dropped");
                let existing = self
                    .windows
                    .get(app_window)
                    .and_then(|s| s.external_drag_token);
                let token = existing.unwrap_or_else(|| self.external_drop.allocate_token());

                let (position, kind, files) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    if state.external_drag_token.is_none() {
                        state.external_drag_token = Some(token);
                    }
                    let position = fret_runner_winit::map_physical_position_to_point(
                        state.window.scale_factor(),
                        position,
                    );
                    if state.external_drag_files.is_empty() {
                        state.external_drag_files = paths;
                    }
                    let files = std::mem::take(&mut state.external_drag_files);
                    state.external_drag_token = None;
                    let kind = ExternalDragKind::DropFiles(fret_runner_winit::external_drag_files(
                        token, &files,
                    ));
                    (position, kind, files)
                };

                self.external_drop.set_payload_paths(token, files);

                self.deliver_window_event_now(
                    app_window,
                    &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                );
                self.drain_effects(event_loop);
            }
            WindowEvent::DragLeft { position } => {
                tracing::debug!("winit drag left");
                let (position, token) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    let position = fret_runner_winit::map_optional_physical_position_to_point(
                        state.window.scale_factor(),
                        position,
                        state.platform.input.cursor_pos,
                    );
                    state.external_drag_files.clear();
                    let token = state.external_drag_token.take();
                    (position, token)
                };

                if let Some(token) = token {
                    self.external_drop.release(token);
                }

                self.deliver_window_event_now(
                    app_window,
                    &Event::ExternalDrag(ExternalDragEvent {
                        position,
                        kind: ExternalDragKind::Leave,
                    }),
                );
                self.drain_effects(event_loop);
            }
            WindowEvent::SurfaceResized(size) => {
                self.sync_surface_resize_now(app_window, size);
                #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
                if macos_hit_test::has_active_regions() {
                    macos_hit_test::apply_latest_mouse_location();
                }
                self.request_surface_resize_redraw(app_window);
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::PointerMoved { .. } => {
                let (mapped, pos, external_drag_token, screen_pos, _scale_factor) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };

                    let mut mapped = Vec::new();
                    state.platform.handle_window_event(
                        state.window.scale_factor(),
                        ev,
                        &mut mapped,
                    );

                    let pos = state.platform.input.cursor_pos;
                    let external_drag_token = state.external_drag_token;
                    let scale_factor = state.window.scale_factor();
                    let screen_pos = match ev {
                        WindowEvent::PointerMoved {
                            position, source, ..
                        } if !matches!(source, winit::event::PointerSource::Touch { .. }) => {
                            state.window.outer_position().ok().map(|outer| {
                                let surface = state.window.surface_position();
                                PhysicalPosition::new(
                                    outer.x as f64 + surface.x as f64 + position.x,
                                    outer.y as f64 + surface.y as f64 + position.y,
                                )
                            })
                        }
                        _ => None,
                    };

                    (mapped, pos, external_drag_token, screen_pos, scale_factor)
                };

                let suppress_os_cursor_sample = self.diag_pointer_input_isolation_active();

                if !suppress_os_cursor_sample {
                    if let Some(p) = screen_pos {
                        self.cursor_screen_pos = Some(p);
                        #[cfg(target_os = "macos")]
                        self.macos_calibrate_cursor_transform_from_window_sample(p, _scale_factor);
                    }

                    let _ = self.update_dock_tearoff_follow();
                }

                let dock_drag_capture = self
                    .dock_drag_pointer_id()
                    .and_then(|pointer_id| {
                        self.app
                            .drag(pointer_id)
                            .map(|d| (pointer_id, d.source_window))
                    })
                    .filter(|(_pointer_id, window)| self.windows.contains_key(*window));

                if let Some(token) = external_drag_token {
                    let paths = self.external_drop.paths(token).unwrap_or(&[]);
                    let kind = ExternalDragKind::OverFiles(fret_runner_winit::external_drag_files(
                        token, paths,
                    ));
                    let evt = Event::ExternalDrag(ExternalDragEvent {
                        position: pos,
                        kind,
                    });
                    self.deliver_window_event_now(app_window, &evt);
                }

                for evt in mapped {
                    let mut delivered = false;
                    if let Some((dock_pointer_id, dock_source_window)) = dock_drag_capture
                        && dock_source_window != app_window
                        && let Event::Pointer(fret_core::PointerEvent::Move {
                            pointer_id,
                            position: _,
                            buttons,
                            modifiers,
                            pointer_type,
                        }) = &evt
                        && *pointer_id == dock_pointer_id
                    {
                        let pos = self
                            .cursor_screen_pos
                            .and_then(|screen| {
                                self.local_pos_for_window(dock_source_window, screen)
                            })
                            .or_else(|| {
                                self.windows
                                    .get(dock_source_window)
                                    .map(|w| w.platform.input.cursor_pos)
                            })
                            .unwrap_or(pos);
                        self.deliver_window_event_now(
                            dock_source_window,
                            &Event::Pointer(fret_core::PointerEvent::Move {
                                pointer_id: *pointer_id,
                                position: pos,
                                buttons: *buttons,
                                modifiers: *modifiers,
                                pointer_type: *pointer_type,
                            }),
                        );
                        delivered = true;
                    }
                    if !delivered {
                        self.deliver_window_event_now(app_window, &evt);
                    }
                }

                self.sync_dock_drag_pointer_capture();
                self.route_internal_drag_hover_from_cursor();
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::PointerButton { .. } => {
                let (mapped, _scale_factor) = {
                    let Some(runtime) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let mut mapped = Vec::new();
                    runtime.platform.handle_window_event(
                        runtime.window.scale_factor(),
                        ev,
                        &mut mapped,
                    );
                    (mapped, runtime.window.scale_factor())
                };

                if let Some(p) = self.cursor_screen_pos_fallback_for_window(app_window) {
                    self.cursor_screen_pos = Some(p);
                    #[cfg(target_os = "macos")]
                    self.macos_calibrate_cursor_transform_from_window_sample(p, _scale_factor);
                }

                self.sync_dock_drag_pointer_capture();

                let dock_drag_capture = self
                    .dock_drag_pointer_id()
                    .and_then(|pointer_id| {
                        self.app
                            .drag(pointer_id)
                            .map(|d| (pointer_id, d.source_window))
                    })
                    .filter(|(_pointer_id, window)| self.windows.contains_key(*window));
                let dock_drag_capture_pos = self.cursor_screen_pos;

                let mut saw_left_down = false;
                let mut saw_left_up = false;
                let mut left_up_pointer_id: Option<fret_core::PointerId> = None;
                for evt in &mapped {
                    let Event::Pointer(pointer) = evt else {
                        continue;
                    };
                    match pointer {
                        fret_core::PointerEvent::Down {
                            button: fret_core::MouseButton::Left,
                            pointer_id: _,
                            ..
                        } => {
                            saw_left_down = true;
                        }
                        fret_core::PointerEvent::Up {
                            button: fret_core::MouseButton::Left,
                            pointer_id,
                            ..
                        } => {
                            saw_left_up = true;
                            left_up_pointer_id = Some(*pointer_id);
                        }
                        _ => {}
                    }
                }

                if saw_left_down {
                    self.left_mouse_down = true;
                }

                if saw_left_up {
                    self.left_mouse_down = false;
                    self.saw_left_mouse_release_this_turn = true;
                    let cancel_pointer_id = self.internal_drag_routing_pointer_id();
                    // Deliver the cursor-based drop on any left mouse release; this keeps docking
                    // robust even when the drag pointer id is not `PointerId(0)`.
                    self.route_internal_drag_drop_from_cursor();
                    if self.dock_tearoff_follow.is_some() {
                        self.stop_dock_tearoff_follow(Instant::now(), true);
                    }

                    // Cross-window drags are runner-routed (Enter/Over/Drop), so ensure the
                    // drag session cannot get "stuck" if no widget ends it.
                    if let Some(released) = cancel_pointer_id.or(left_up_pointer_id)
                        && self
                            .app
                            .drag(released)
                            .is_some_and(|d| d.cross_window_hover)
                    {
                        self.app.cancel_drag(released);
                        let _ = self.clear_internal_drag_hover_if_needed();
                    }
                }

                for evt in mapped {
                    let mut delivered = false;
                    if let Some((dock_pointer_id, dock_source_window)) = dock_drag_capture
                        && dock_source_window != app_window
                    {
                        match &evt {
                            Event::Pointer(fret_core::PointerEvent::Up {
                                pointer_id,
                                position: _,
                                button,
                                modifiers,
                                is_click,
                                click_count,
                                pointer_type,
                            }) if *pointer_id == dock_pointer_id => {
                                let pos = dock_drag_capture_pos
                                    .and_then(|screen| {
                                        self.local_pos_for_window(dock_source_window, screen)
                                    })
                                    .or_else(|| {
                                        self.windows
                                            .get(dock_source_window)
                                            .map(|w| w.platform.input.cursor_pos)
                                    })
                                    .unwrap_or_default();
                                self.deliver_window_event_now(
                                    dock_source_window,
                                    &Event::Pointer(fret_core::PointerEvent::Up {
                                        pointer_id: *pointer_id,
                                        position: pos,
                                        button: *button,
                                        modifiers: *modifiers,
                                        is_click: *is_click,
                                        click_count: *click_count,
                                        pointer_type: *pointer_type,
                                    }),
                                );
                                delivered = true;
                            }
                            Event::Pointer(fret_core::PointerEvent::Down {
                                pointer_id,
                                position: _,
                                button,
                                modifiers,
                                click_count,
                                pointer_type,
                            }) if *pointer_id == dock_pointer_id => {
                                let pos = dock_drag_capture_pos
                                    .and_then(|screen| {
                                        self.local_pos_for_window(dock_source_window, screen)
                                    })
                                    .or_else(|| {
                                        self.windows
                                            .get(dock_source_window)
                                            .map(|w| w.platform.input.cursor_pos)
                                    })
                                    .unwrap_or_default();
                                self.deliver_window_event_now(
                                    dock_source_window,
                                    &Event::Pointer(fret_core::PointerEvent::Down {
                                        pointer_id: *pointer_id,
                                        position: pos,
                                        button: *button,
                                        modifiers: *modifiers,
                                        click_count: *click_count,
                                        pointer_type: *pointer_type,
                                    }),
                                );
                                delivered = true;
                            }
                            _ => {}
                        }
                    }
                    if !delivered {
                        self.deliver_window_event_now(app_window, &evt);
                    }
                }
                self.drain_effects(event_loop);
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
                let mapped = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let mut mapped = Vec::new();
                    state.platform.handle_window_event(
                        state.window.scale_factor(),
                        ev,
                        &mut mapped,
                    );
                    mapped
                };

                let wheel_coalescing_enabled = Self::wheel_coalescing_enabled();
                let mut saw_wheel = false;
                let mapped = if wheel_coalescing_enabled {
                    let mut passthrough = Vec::with_capacity(mapped.len());
                    let mut pending: Option<PendingWheelEvent> = None;

                    for evt in mapped {
                        match evt {
                            Event::Pointer(fret_core::PointerEvent::Wheel {
                                pointer_id,
                                position,
                                delta,
                                modifiers,
                                pointer_type,
                            }) => {
                                saw_wheel = true;
                                pending = Some(match pending {
                                    Some(mut prev) => {
                                        prev.delta = wheel_coalesce_delta(prev.delta, delta);
                                        prev.position = position;
                                        prev.modifiers = modifiers;
                                        prev.pointer_type = pointer_type;
                                        prev.pointer_id = pointer_id;
                                        prev
                                    }
                                    None => PendingWheelEvent {
                                        pointer_id,
                                        position,
                                        delta,
                                        modifiers,
                                        pointer_type,
                                    },
                                });
                            }
                            other => passthrough.push(other),
                        }
                    }

                    if let Some(pending) = pending
                        && let Some(state) = self.windows.get_mut(app_window)
                    {
                        state.pending_wheel = Some(match state.pending_wheel.take() {
                            Some(mut prev) => {
                                prev.delta = wheel_coalesce_delta(prev.delta, pending.delta);
                                prev.position = pending.position;
                                prev.modifiers = pending.modifiers;
                                prev.pointer_type = pending.pointer_type;
                                prev.pointer_id = pending.pointer_id;
                                prev
                            }
                            None => pending,
                        });
                    }

                    passthrough
                } else {
                    mapped
                };

                if saw_wheel {
                    self.app.request_redraw(app_window);
                }

                if mapped.iter().any(|evt| {
                    matches!(
                        evt,
                        Event::KeyDown {
                            key: fret_core::KeyCode::F12,
                            ..
                        }
                    )
                }) {
                    if let Some(r) = self.renderdoc.as_mut() {
                        r.request_capture();
                        self.app.request_redraw(app_window);
                    } else if std::env::var_os("FRET_RENDERDOC")
                        .filter(|v| !v.is_empty())
                        .is_some()
                        || std::env::var_os("FRET_RENDERDOC_DLL")
                            .filter(|v| !v.is_empty())
                            .is_some()
                    {
                        tracing::warn!(
                            "renderdoc capture requested but renderdoc was not initialized (restart with renderdoc.dll available)"
                        );
                    }
                }

                // ADR 0072 (proposed): Escape cancels an active cross-window dock drag session.
                if mapped.iter().any(|evt| {
                    matches!(
                        evt,
                        Event::KeyDown {
                            key: fret_core::KeyCode::Escape,
                            ..
                        }
                    )
                }) && self.dock_drag_pointer_id().is_some()
                {
                    if let Some(pointer_id) = self.dock_drag_pointer_id() {
                        self.app.cancel_drag(pointer_id);
                    }
                    let _ = self.clear_internal_drag_hover_if_needed();
                    if self.dock_tearoff_follow.is_some() {
                        self.stop_dock_tearoff_follow(Instant::now(), true);
                    }
                    self.drain_effects(event_loop);
                    return;
                }

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }
                self.drain_effects(event_loop);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        // Ensure effects requested during `RedrawRequested` (after the pre-render drain) are still
        // observed before the loop sleeps (e.g. `App::request_redraw()` inside a render callback).
        self.drain_effects(event_loop);

        if self.is_suspended {
            event_loop.set_control_flow(ControlFlow::Wait);
            return;
        }

        self.refresh_runner_monitor_topology_diagnostics(event_loop);

        #[cfg(feature = "diag-screenshots")]
        let pending_diag_screenshot_windows: Vec<fret_core::AppWindowId> =
            if let Some(diag) = self.diag_screenshots.as_mut() {
                diag.poll();
                self.windows
                    .keys()
                    .filter(|window| diag.has_pending_for_window(window.data().as_ffi()))
                    .collect()
            } else {
                Vec::new()
            };
        #[cfg(feature = "diag-screenshots")]
        for window in pending_diag_screenshot_windows {
            let _ = self.request_window_redraw_with_reason(
                window,
                fret_runtime::RunnerFrameDriveReason::EffectRedraw,
            );
            self.raf_windows.request(window);
        }

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            // Only attempt to (re)create missing surfaces after winit has indicated surfaces may
            // be created for this lifecycle turn. Calling the `can_create_surfaces` hook
            // directly would bypass the winit gate and can fail early on Android.
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

        self.handle_about_to_wait_internal_drag_poll(event_loop);
        self.tick_id = scheduling::begin_turn(&mut self.tick_id);
        self.app.set_tick_id(self.tick_id);
        self.saw_left_mouse_release_this_turn = false;
        let now = Instant::now();
        self.poll_window_environment_if_due(now);

        #[cfg(all(
            feature = "dev-state",
            not(any(target_os = "android", target_os = "ios"))
        ))]
        {
            if self.dev_state.enabled() {
                let alive: std::collections::HashSet<fret_core::AppWindowId> =
                    self.windows.keys().collect();
                self.dev_state
                    .sync_window_keys_from_app(&self.app, |window| alive.contains(&window));
                self.dev_state.export_app_state(&mut self.app);
                let keys = self.dev_state.window_keys_snapshot();
                let mut observed: Vec<(
                    String,
                    winit::dpi::LogicalSize<f64>,
                    Option<winit::dpi::PhysicalPosition<i32>>,
                )> = Vec::new();
                for (window, key) in keys {
                    let Some(state) = self.windows.get(window) else {
                        continue;
                    };
                    let physical = state.window.surface_size();
                    let logical: winit::dpi::LogicalSize<f64> =
                        physical.to_logical(state.window.scale_factor());
                    let position = state.window.outer_position().ok();
                    observed.push((key, logical, position));
                }
                self.dev_state.observe_windows(now, &self.app, observed);
            }
        }

        #[cfg(target_os = "ios")]
        if self.ios_keyboard.is_none() {
            self.ios_keyboard = Some(ios_keyboard::IosKeyboardTracker::new());
        }

        for (app_window, state) in self.windows.iter_mut() {
            #[cfg(target_os = "android")]
            {
                use winit::platform::android::WindowExtAndroid as _;

                let content_rect = state.window.content_rect();
                let surface_size = state.window.surface_size();
                let scale_factor = (state.window.scale_factor() as f32).max(0.0001);

                let surface_w = surface_size.width as i32;
                let surface_h = surface_size.height as i32;

                let left_px = content_rect.left.max(0).min(surface_w) as f32;
                let top_px = content_rect.top.max(0).min(surface_h) as f32;
                let right_px = (surface_w - content_rect.right).max(0).min(surface_w) as f32;
                let bottom_px = (surface_h - content_rect.bottom).max(0).min(surface_h) as f32;

                let focus_is_text_input = self
                    .app
                    .global::<fret_runtime::WindowTextInputSnapshotService>()
                    .and_then(|svc| svc.snapshot(app_window))
                    .map(|s| s.focus_is_text_input)
                    .unwrap_or(false);

                let bottom_inset = Px(bottom_px / scale_factor);
                let baseline_bottom_inset = match state.android_bottom_inset_baseline {
                    Some(prev) if focus_is_text_input => Px(prev.0.min(bottom_inset.0)),
                    _ => bottom_inset,
                };
                state.android_bottom_inset_baseline = Some(baseline_bottom_inset);

                let ime_bottom_inset = if focus_is_text_input {
                    Px((bottom_inset.0 - baseline_bottom_inset.0).max(0.0))
                } else {
                    Px(0.0)
                };

                let safe_area_insets = fret_core::Edges {
                    top: Px(top_px / scale_factor),
                    right: Px(right_px / scale_factor),
                    bottom: baseline_bottom_inset,
                    left: Px(left_px / scale_factor),
                };
                let occlusion_insets = fret_core::Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: ime_bottom_inset,
                    left: Px(0.0),
                };

                let overrides = self.diag_window_insets_overrides.get(&app_window);
                let safe_area_insets = overrides
                    .and_then(|ovr| ovr.safe_area_insets.clone())
                    .unwrap_or(Some(safe_area_insets));
                let occlusion_insets = overrides
                    .and_then(|ovr| ovr.occlusion_insets.clone())
                    .unwrap_or(Some(occlusion_insets));

                let mut insets_changed = false;
                self.app
                    .with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
                        if svc.safe_area_insets(app_window) != safe_area_insets {
                            svc.set_safe_area_insets(app_window, safe_area_insets);
                            insets_changed = true;
                        }
                        if svc.occlusion_insets(app_window) != occlusion_insets {
                            svc.set_occlusion_insets(app_window, occlusion_insets);
                            insets_changed = true;
                        }
                    });
                if insets_changed {
                    state.window.request_redraw();
                }
            }

            #[cfg(target_os = "ios")]
            {
                let safe_area = state.window.safe_area();
                let scale_factor = (state.window.scale_factor() as f32).max(0.0001);

                let safe_area_insets = fret_core::Edges {
                    top: Px(safe_area.top as f32 / scale_factor),
                    right: Px(safe_area.right as f32 / scale_factor),
                    bottom: Px(safe_area.bottom as f32 / scale_factor),
                    left: Px(safe_area.left as f32 / scale_factor),
                };

                let focus_is_text_input = self
                    .app
                    .global::<fret_runtime::WindowTextInputSnapshotService>()
                    .and_then(|svc| svc.snapshot(app_window))
                    .map(|s| s.focus_is_text_input)
                    .unwrap_or(false);

                let keyboard_overlap_bottom = if focus_is_text_input {
                    let frame = self
                        .ios_keyboard
                        .as_ref()
                        .and_then(|tracker| tracker.keyboard_frame_screen());
                    frame
                        .and_then(|frame| {
                            ios_keyboard::keyboard_overlap_bottom_in_window_points(
                                &*state.window,
                                frame,
                            )
                        })
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let ime_bottom_inset =
                    Px((keyboard_overlap_bottom - safe_area_insets.bottom.0).max(0.0));

                let occlusion_insets = fret_core::Edges {
                    top: Px(0.0),
                    right: Px(0.0),
                    bottom: ime_bottom_inset,
                    left: Px(0.0),
                };

                let overrides = self.diag_window_insets_overrides.get(&app_window);
                let safe_area_insets = overrides
                    .and_then(|ovr| ovr.safe_area_insets.clone())
                    .unwrap_or(Some(safe_area_insets));
                let occlusion_insets = overrides
                    .and_then(|ovr| ovr.occlusion_insets.clone())
                    .unwrap_or(Some(occlusion_insets));

                let mut insets_changed = false;
                self.app
                    .with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
                        if svc.safe_area_insets(app_window) != safe_area_insets {
                            svc.set_safe_area_insets(app_window, safe_area_insets);
                            insets_changed = true;
                        }
                        if svc.occlusion_insets(app_window) != occlusion_insets {
                            svc.set_occlusion_insets(app_window, occlusion_insets);
                            insets_changed = true;
                        }
                    });
                if insets_changed {
                    state.window.request_redraw();
                }
            }

            let Some(a11y) = state.accessibility.as_mut() else {
                continue;
            };

            if a11y.take_activation_request() {
                self.app.with_global_mut(
                    fret_runtime::RunnerAccessibilityDiagnosticsStore::default,
                    |store, app| {
                        store.record_activation_request(app_window, app.frame_id());
                    },
                );
                state.window.request_redraw();
            }

            let mut requests = Vec::new();
            a11y.drain_actions(&mut requests);
            a11y.drain_actions_fallback(&mut requests);

            for req in requests {
                if let Some(target) = accessibility::focus_target_from_action(&req) {
                    self.driver.accessibility_focus(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        target,
                    );
                    self.app.request_redraw(app_window);
                    continue;
                }

                if let Some(target) = accessibility::invoke_target_from_action(&req) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.accessibility_invoke(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        target,
                    );
                    self.app.request_redraw(app_window);
                    continue;
                }

                if let Some((target, action)) = accessibility::stepper_target_from_action(&req) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    match action {
                        accessibility::StepperAction::Decrement => {
                            self.driver.accessibility_decrement(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                            );
                        }
                        accessibility::StepperAction::Increment => {
                            self.driver.accessibility_increment(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                            );
                        }
                    }
                    self.app.request_redraw(app_window);
                    continue;
                }

                if let Some((target, data)) = accessibility::set_value_from_action(&req) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    match data {
                        accessibility::SetValueData::Text(value) => {
                            self.driver.accessibility_set_value_text(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                                &value,
                            );
                        }
                        accessibility::SetValueData::Numeric(value) => {
                            self.driver.accessibility_set_value_numeric(
                                &mut self.app,
                                services,
                                app_window,
                                &mut state.user,
                                target,
                                value,
                            );
                        }
                    }
                    self.app.request_redraw(app_window);
                    continue;
                }

                let snapshot = state.last_semantics_snapshot.clone().or_else(|| {
                    self.driver
                        .semantics_snapshot(&mut self.app, app_window, &mut state.user)
                });
                if let Some(snapshot) = snapshot {
                    if let Some((target, data)) =
                        accessibility::scroll_by_from_action(&req, &snapshot)
                    {
                        self.driver.accessibility_scroll_by(
                            &mut self.app,
                            app_window,
                            &mut state.user,
                            target,
                            data.dx,
                            data.dy,
                        );
                        self.app.request_redraw(app_window);
                        continue;
                    }

                    if let Some((target, value)) =
                        accessibility::replace_selected_text_from_action(&req, &snapshot)
                    {
                        let services =
                            Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                        self.driver.accessibility_replace_selected_text(
                            &mut self.app,
                            services,
                            app_window,
                            &mut state.user,
                            target,
                            &value,
                        );
                        self.app.request_redraw(app_window);
                        continue;
                    }

                    if let Some((target, data)) =
                        accessibility::set_text_selection_from_action(&req, &snapshot)
                    {
                        let services =
                            Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                        self.driver.accessibility_set_text_selection(
                            &mut self.app,
                            services,
                            app_window,
                            &mut state.user,
                            target,
                            data.anchor,
                            data.focus,
                        );
                        self.app.request_redraw(app_window);
                        continue;
                    }
                }
            }
        }

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
