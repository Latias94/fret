use super::*;

impl<D: WinitDriver> ApplicationHandler for WinitRunner<D> {
    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<winit::event::DeviceId>,
        event: DeviceEvent,
    ) {
        if !self.app.drag().is_some_and(|d| d.cross_window_hover)
            && self.dock_tearoff_follow.is_none()
        {
            return;
        }

        match event {
            DeviceEvent::PointerMotion { delta } => {
                #[cfg(target_os = "windows")]
                {
                    if let Some(p) = win32::cursor_pos_physical() {
                        self.cursor_screen_pos = Some(p);
                    } else {
                        let Some(pos) = self.cursor_screen_pos else {
                            return;
                        };
                        self.cursor_screen_pos =
                            Some(PhysicalPosition::new(pos.x + delta.0, pos.y + delta.1));
                    }
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let Some(pos) = self.cursor_screen_pos else {
                        return;
                    };

                    self.cursor_screen_pos =
                        Some(PhysicalPosition::new(pos.x + delta.0, pos.y + delta.1));
                }
                self.route_internal_drag_hover_from_cursor();
                let _ = self.update_dock_tearoff_follow();
                self.drain_effects(event_loop);
            }
            DeviceEvent::Button {
                state: ElementState::Released,
                ..
            } => {
                #[cfg(target_os = "windows")]
                if let Some(p) = win32::cursor_pos_physical() {
                    self.cursor_screen_pos = Some(p);
                }

                // This fallback path is only for releases that occur outside all windows, where
                // winit may not emit `WindowEvent::MouseInput`. When releasing over any window,
                // prefer the regular window event path; otherwise we can incorrectly "force tear-off"
                // even when the user is trying to dock back into another window.
                if let Some(pos) = self.cursor_screen_pos
                    && self.window_under_cursor(pos, None).is_some()
                {
                    return;
                }

                // On macOS, releasing the mouse button outside any window may not deliver a
                // `WindowEvent::MouseInput` to the source window. Use device events to still
                // terminate cross-window dock drags (Unity/ImGui-style tear-off).
                let (source_window, current_window, dragging) = {
                    let Some(drag) = self.app.drag() else {
                        return;
                    };
                    if drag.kind != fret_app::DragKind::DockPanel {
                        return;
                    }
                    (drag.source_window, drag.current_window, drag.dragging)
                };
                dock_tearoff_log(format_args!(
                    "[device-up] source={:?} current={:?} screen_pos={:?} dragging={}",
                    source_window, current_window, self.cursor_screen_pos, dragging
                ));

                #[cfg(target_os = "macos")]
                {
                    if self.saw_left_mouse_release_this_turn || macos_is_left_mouse_down() {
                        return;
                    }
                    if let Some(d) = self.app.drag_mut()
                        && d.kind == fret_app::DragKind::DockPanel
                    {
                        d.dragging = true;
                    }
                    // Route the drop using the current cursor position, so docking into another
                    // window works even when the `MouseInput` event is missing.
                    self.route_internal_drag_drop_from_cursor();
                    dock_tearoff_log(format_args!(
                        "[device-drop] dispatched target={:?}",
                        source_window
                    ));
                }
                if self.app.drag().is_some_and(|d| d.cross_window_hover) {
                    self.app.cancel_drag();
                    let _ = self.clear_internal_drag_hover_if_needed();
                }
                // When a floating dock window is following the cursor, a mouse release may occur
                // outside any window and never produce `WindowEvent::MouseInput`.
                if self.dock_tearoff_follow.is_some() {
                    self.left_mouse_down = false;
                    self.stop_dock_tearoff_follow(Instant::now(), true);
                }
                self.drain_effects(event_loop);
            }
            _ => {}
        }
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.context.is_some() {
            return;
        }

        let spec = self.config.main_window_spec();
        let window = match self.create_os_window(event_loop, spec) {
            Ok(w) => w,
            Err(e) => {
                error!(error = ?e, "failed to create main window");
                return;
            }
        };

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
                            error!(error = ?e, "failed to create surface from provided context");
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
        let mut renderer = Renderer::new(&context.adapter, &context.device);
        renderer.set_svg_raster_budget_bytes(self.config.svg_raster_budget_bytes);
        renderer.set_path_msaa_samples(self.config.path_msaa_samples);
        let _ = renderer.set_text_font_families(&self.config.text_font_families);
        self.app
            .set_global::<fret_core::TextFontFamilyConfig>(self.config.text_font_families.clone());
        self.app
            .set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
                renderer.text_font_stack_key(),
            ));

        let prev_rev = self
            .app
            .global::<fret_runtime::FontCatalog>()
            .map(|c| c.revision)
            .unwrap_or(0);
        let revision = prev_rev.saturating_add(1);
        let families = renderer.all_font_names();
        let cache = fret_app::FontCatalogCache::from_families(revision, &families);
        self.app
            .set_global::<fret_runtime::FontCatalog>(fret_runtime::FontCatalog {
                families,
                revision,
            });
        self.app.set_global::<fret_app::FontCatalogCache>(cache);

        self.context = Some(context);
        self.renderer = Some(renderer);
        if let (Some(context), Some(renderer)) = (self.context.as_ref(), self.renderer.as_mut()) {
            self.driver.gpu_ready(&mut self.app, context, renderer);
        }

        let main_window = match self.insert_window(window.0, window.1, surface) {
            Ok(id) => id,
            Err(e) => {
                error!(error = ?e, "failed to insert main window runtime");
                return;
            }
        };
        self.main_window = Some(main_window);
        self.driver.init(&mut self.app, main_window);
        self.app.request_redraw(main_window);
        self.drain_effects(event_loop);
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        let pending = self
            .proxy_events
            .lock()
            .ok()
            .map(|mut q| std::mem::take(&mut *q))
            .unwrap_or_default();

        for event in pending {
            match event {
                RunnerUserEvent::PlatformCompletion { window, completion } => {
                    self.deliver_platform_completion_now(window, completion);
                }
            }
        }

        self.drain_effects(event_loop);
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

        match event {
            WindowEvent::CloseRequested => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowCloseRequested,
                    );
                }
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::ModifiersChanged(..) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.platform.handle_window_event(
                        state.window.scale_factor(),
                        ev,
                        &mut Vec::new(),
                    );
                }

                if self.app.drag().is_some_and(|d| {
                    d.cross_window_hover && d.kind == fret_app::DragKind::DockPanel
                }) {
                    self.route_internal_drag_hover_from_cursor();
                    self.drain_effects(event_loop);
                }
            }
            WindowEvent::Focused(focused) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.is_focused = focused;
                    if !focused {
                        state.platform.input.pressed_buttons = fret_core::MouseButtons::default();
                    }
                }
                macos_window_log(format_args!(
                    "[focused] app_window={:?} focused={} winit={:?}",
                    app_window, focused, window_id
                ));
            }
            ref ev @ WindowEvent::Moved(_) => {
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

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::KeyboardInput { ref event, .. } => {
                let key = fret_runner_winit::map_physical_key(event.physical_key);

                // ADR 0072 (proposed): Escape cancels an active cross-window dock drag session.
                if event.state == ElementState::Pressed
                    && key == fret_core::KeyCode::Escape
                    && self.app.drag().is_some_and(|d| {
                        d.cross_window_hover && d.kind == fret_app::DragKind::DockPanel
                    })
                {
                    self.app.cancel_drag();
                    let _ = self.clear_internal_drag_hover_if_needed();
                    if self.dock_tearoff_follow.is_some() {
                        self.stop_dock_tearoff_follow(Instant::now(), true);
                    }
                    self.drain_effects(event_loop);
                    return;
                }

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

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::Ime(ime) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    if std::env::var_os("FRET_IME_DEBUG").is_some_and(|v| !v.is_empty()) {
                        tracing::info!(
                            "IME_DEBUG winit: WindowEvent::Ime({:?}) cached_rect={}",
                            ime,
                            state.platform.ime_cursor_area().is_some()
                        );
                    }
                    #[cfg(windows)]
                    if let Some(rect) = state.platform.ime_cursor_area() {
                        fret_runner_winit::windows_ime::set_ime_cursor_area(
                            state.window.as_ref(),
                            rect,
                        );
                    }

                    let mapped = match ime {
                        winit::event::Ime::Enabled => fret_core::ImeEvent::Enabled,
                        winit::event::Ime::Disabled => fret_core::ImeEvent::Disabled,
                        winit::event::Ime::Commit(text) => fret_core::ImeEvent::Commit(text),
                        winit::event::Ime::Preedit(text, cursor) => {
                            fret_core::ImeEvent::Preedit { text, cursor }
                        }
                        winit::event::Ime::DeleteSurrounding { .. } => {
                            self.drain_effects(event_loop);
                            return;
                        }
                    };
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::Ime(mapped),
                    );
                }
                self.drain_effects(event_loop);
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
                    let logical: winit::dpi::LogicalPosition<f32> =
                        position.to_logical(state.window.scale_factor());
                    let position = Point::new(Px(logical.x), Px(logical.y));
                    state.external_drag_files = paths;
                    let files = state.external_drag_files.clone();
                    let kind =
                        ExternalDragKind::EnterFiles(Self::external_drag_files(token, &files));
                    (position, kind, files)
                };

                self.external_drop.set_payload_paths(token, files);

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::DragMoved { position } => {
                let (position, token) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    let logical: winit::dpi::LogicalPosition<f32> =
                        position.to_logical(state.window.scale_factor());
                    let position = Point::new(Px(logical.x), Px(logical.y));
                    (position, state.external_drag_token)
                };

                if let Some(token) = token {
                    let paths = self.external_drop.paths(token).unwrap_or(&[]);
                    let kind = ExternalDragKind::OverFiles(Self::external_drag_files(token, paths));
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
                    let logical: winit::dpi::LogicalPosition<f32> =
                        position.to_logical(state.window.scale_factor());
                    let position = Point::new(Px(logical.x), Px(logical.y));
                    if state.external_drag_files.is_empty() {
                        state.external_drag_files = paths;
                    }
                    let files = std::mem::take(&mut state.external_drag_files);
                    state.external_drag_token = None;
                    let kind =
                        ExternalDragKind::DropFiles(Self::external_drag_files(token, &files));
                    (position, kind, files)
                };

                self.external_drop.set_payload_paths(token, files);

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent { position, kind }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::DragLeft { position } => {
                tracing::debug!("winit drag left");
                let (position, token) = {
                    let Some(state) = self.windows.get_mut(app_window) else {
                        self.drain_effects(event_loop);
                        return;
                    };
                    let position = if let Some(position) = position {
                        let logical: winit::dpi::LogicalPosition<f32> =
                            position.to_logical(state.window.scale_factor());
                        Point::new(Px(logical.x), Px(logical.y))
                    } else {
                        state.platform.input.cursor_pos
                    };
                    state.external_drag_files.clear();
                    let token = state.external_drag_token.take();
                    (position, token)
                };

                if let Some(token) = token {
                    self.external_drop.release(token);
                }

                if let Some(state) = self.windows.get_mut(app_window) {
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::ExternalDrag(ExternalDragEvent {
                            position,
                            kind: ExternalDragKind::Leave,
                        }),
                    );
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::SurfaceResized(size) => {
                self.resize_surface(app_window, size.width, size.height);
                if let Some(state) = self.windows.get_mut(app_window) {
                    let scale = state.window.scale_factor() as f32;
                    let logical: winit::dpi::LogicalSize<f32> = size.to_logical(scale as f64);
                    self.app
                        .with_global_mut(WindowMetricsService::default, |svc, _app| {
                            svc.set_inner_size(
                                app_window,
                                Size::new(Px(logical.width), Px(logical.height)),
                            );
                        });
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowResized {
                            width: Px(logical.width),
                            height: Px(logical.height),
                        },
                    );
                    let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
                    self.driver.handle_event(
                        &mut self.app,
                        services,
                        app_window,
                        &mut state.user,
                        &Event::WindowScaleFactorChanged(scale),
                    );
                }
                self.app.request_redraw(app_window);
            }
            ref ev @ WindowEvent::PointerMoved { position, .. } => {
                let (mapped, pos, external_drag_token, screen_pos) = {
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
                    let screen_pos = state.window.outer_position().ok().map(|outer| {
                        let surface = state.window.surface_position();
                        PhysicalPosition::new(
                            outer.x as f64 + surface.x as f64 + position.x,
                            outer.y as f64 + surface.y as f64 + position.y,
                        )
                    });

                    (mapped, pos, external_drag_token, screen_pos)
                };

                if let Some(p) = screen_pos {
                    self.cursor_screen_pos = Some(p);
                }

                let _ = self.update_dock_tearoff_follow();

                if let Some(token) = external_drag_token {
                    let paths = self.external_drop.paths(token).unwrap_or(&[]);
                    let kind = ExternalDragKind::OverFiles(Self::external_drag_files(token, paths));
                    let evt = Event::ExternalDrag(ExternalDragEvent {
                        position: pos,
                        kind,
                    });
                    self.deliver_window_event_now(app_window, &evt);
                }

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }

                self.route_internal_drag_hover_from_cursor();
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::PointerButton {
                state, ref button, ..
            } => {
                let mapped = {
                    let Some(runtime) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let mut mapped = Vec::new();
                    runtime.platform.handle_window_event(
                        runtime.window.scale_factor(),
                        ev,
                        &mut mapped,
                    );
                    mapped
                };

                let Some(winit_button) = fret_runner_winit::map_pointer_button(button) else {
                    return;
                };
                let button = fret_runner_winit::map_mouse_button(winit_button);

                match state {
                    ElementState::Pressed => {
                        if button == fret_core::MouseButton::Left {
                            self.left_mouse_down = true;
                        }
                    }
                    ElementState::Released => {
                        if button == fret_core::MouseButton::Left {
                            self.left_mouse_down = false;
                            self.saw_left_mouse_release_this_turn = true;
                            self.route_internal_drag_drop_from_cursor();
                            self.stop_dock_tearoff_follow(Instant::now(), true);
                            // Cross-window drags are runner-routed (Enter/Over/Drop), so ensure the
                            // drag session cannot get "stuck" if no widget ends it.
                            if self.app.drag().is_some_and(|d| d.cross_window_hover) {
                                self.app.cancel_drag();
                                let _ = self.clear_internal_drag_hover_if_needed();
                            }
                        }
                    }
                }

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }
                self.drain_effects(event_loop);
            }
            ref ev @ WindowEvent::MouseWheel { .. } => {
                let mapped = {
                    let Some(runtime) = self.windows.get_mut(app_window) else {
                        return;
                    };
                    let mut mapped = Vec::new();
                    runtime.platform.handle_window_event(
                        runtime.window.scale_factor(),
                        ev,
                        &mut mapped,
                    );
                    mapped
                };

                for evt in mapped {
                    self.deliver_window_event_now(app_window, &evt);
                }
                self.drain_effects(event_loop);
            }
            WindowEvent::RedrawRequested => {
                // Drain effects before rendering so dock ops, invalidation bumps, and window
                // requests apply deterministically to the frame being drawn (ADR 0013).
                self.drain_effects(event_loop);

                {
                    let (Some(context), Some(renderer)) =
                        (self.context.as_ref(), self.renderer.as_mut())
                    else {
                        return;
                    };
                    let Some(state) = self.windows.get_mut(app_window) else {
                        return;
                    };

                    // Apply any pending window-side state (IME/cursor) once per frame, similar to
                    // Dear ImGui's backend `prepare_frame` pattern.
                    state.platform.prepare_frame(state.window.as_ref());

                    let scale_factor = state.window.scale_factor() as f32;
                    let physical = state.window.surface_size();
                    let logical: winit::dpi::LogicalSize<f32> =
                        physical.to_logical(state.window.scale_factor());

                    let bounds = Rect::new(
                        Point::new(Px(0.0), Px(0.0)),
                        Size::new(Px(logical.width), Px(logical.height)),
                    );

                    self.driver.gpu_frame_prepare(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        context,
                        renderer,
                        scale_factor,
                    );

                    self.driver.render(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        bounds,
                        scale_factor,
                        renderer as &mut dyn fret_core::UiServices,
                        &mut state.scene,
                    );

                    validate_scene_if_enabled(&state.scene);

                    if let Some(a11y) = state.accessibility.as_mut()
                        && a11y.is_active()
                        && let Some(snapshot) = self.driver.accessibility_snapshot(
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
                        state.last_accessibility_snapshot = Some(snapshot);
                    } else {
                        state.last_accessibility_snapshot = None;
                    }

                    let engine_frame = self.driver.record_engine_frame(
                        &mut self.app,
                        app_window,
                        &mut state.user,
                        context,
                        renderer,
                        scale_factor,
                        self.tick_id,
                        self.frame_id,
                    );

                    for update in engine_frame.target_updates {
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

                    let draw_result = state.surface.present_with(&context.queue, |view| {
                        let ui_cmd = renderer.render_scene(
                            &context.device,
                            &context.queue,
                            fret_render::RenderSceneParams {
                                format: state.surface.format(),
                                target_view: view,
                                scene: &state.scene,
                                clear: self.config.clear_color,
                                scale_factor,
                                viewport_size: state.surface.size(),
                            },
                        );

                        let mut cmd_buffers = engine_frame.command_buffers;
                        cmd_buffers.push(ui_cmd);
                        cmd_buffers
                    });

                    if let Err(err) = draw_result {
                        match err {
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::Lost,
                            } => {
                                let size = state.window.surface_size();
                                self.resize_surface(app_window, size.width, size.height);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::OutOfMemory,
                            } => {
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

                    self.frame_id.0 = self.frame_id.0.saturating_add(1);
                    self.app.set_frame_id(self.frame_id);
                }

                // Drain effects produced during rendering so they don't lag by a frame (e.g. IME
                // cursor updates, timer-driven docking invalidations, window raise/create effects).
                self.drain_effects(event_loop);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.saw_left_mouse_release_this_turn = false;

        for (app_window, state) in self.windows.iter_mut() {
            let Some(a11y) = state.accessibility.as_mut() else {
                continue;
            };

            if a11y.take_activation_request() {
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

                let snapshot = state.last_accessibility_snapshot.clone().or_else(|| {
                    self.driver
                        .accessibility_snapshot(&mut self.app, app_window, &mut state.user)
                });
                if let Some(snapshot) = snapshot {
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

        if let Some(follow) = self.dock_tearoff_follow
            && !self.is_left_mouse_down_for_window(follow.source_window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }

        self.drain_effects(event_loop);

        let now = Instant::now();

        #[cfg(target_os = "macos")]
        {
            if self.maybe_finish_dock_drag_released_outside() {
                self.drain_effects(event_loop);
            }
        }

        let did_pending_front_work = self.process_pending_front_requests(now);

        let mut next_deadline: Option<Instant> = None;
        for entry in self.timers.values() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(entry.deadline),
                None => entry.deadline,
            });
        }

        if let Some(deadline) = self.next_pending_front_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        let drag_poll = self.app.drag().is_some_and(|d| d.cross_window_hover);
        let follow_poll = self.dock_tearoff_follow.is_some();
        let wants_poll = drag_poll || follow_poll;

        let wants_raf = !self.raf_windows.is_empty();
        self.raf_windows.clear();

        let next = match (next_deadline, wants_raf) {
            (Some(deadline), true) => Some((now + self.config.frame_interval).min(deadline)),
            (Some(deadline), false) => Some(deadline),
            (None, true) => Some(now + self.config.frame_interval),
            (None, false) => None,
        };

        if wants_poll {
            event_loop.set_control_flow(ControlFlow::Poll);
        } else if let Some(next) = next {
            event_loop.set_control_flow(ControlFlow::WaitUntil(next));
        } else if did_pending_front_work {
            // Ensure we keep turning the event loop while we try to raise a window on macOS.
            event_loop.set_control_flow(ControlFlow::WaitUntil(now + Duration::from_millis(16)));
        } else {
            event_loop.set_control_flow(ControlFlow::Wait);
        }
    }
}
