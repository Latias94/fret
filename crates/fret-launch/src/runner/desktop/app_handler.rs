//! Winit `ApplicationHandler` integration.

use super::*;
use std::sync::{Mutex, OnceLock};

#[cfg(feature = "diag-screenshots")]
use slotmap::Key as _;

#[derive(Debug, Clone, Copy)]
struct RedrawHitchConfig {
    hitch_ms: u64,
}

fn redraw_hitch_config() -> Option<RedrawHitchConfig> {
    static CONFIG: OnceLock<Option<RedrawHitchConfig>> = OnceLock::new();
    *CONFIG.get_or_init(|| {
        let enabled = std::env::var_os("FRET_REDRAW_HITCH_LOG").is_some_and(|v| !v.is_empty());
        if !enabled {
            return None;
        }

        let hitch_ms = std::env::var("FRET_REDRAW_HITCH_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(24);

        Some(RedrawHitchConfig { hitch_ms })
    })
}

fn redraw_hitch_log_paths() -> impl Iterator<Item = std::path::PathBuf> {
    let mut paths = Vec::new();

    if let Some(custom) = std::env::var_os("FRET_REDRAW_HITCH_LOG_PATH")
        && !custom.is_empty()
    {
        let mut path = std::path::PathBuf::from(custom);
        if path.is_relative()
            && let Some(diag_dir) = std::env::var_os("FRET_DIAG_DIR")
            && !diag_dir.is_empty()
        {
            path = std::path::Path::new(&diag_dir).join(path);
        }
        paths.push(path);
    }

    paths.push(std::path::Path::new(".fret").join("redraw_hitches.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("redraw_hitches.log"));
    }
    paths.into_iter()
}

fn quantize_logical_px(value: f32) -> f32 {
    if !value.is_finite() || value <= 0.0 {
        return 0.0;
    }
    let quantum = 64.0f32;
    (value * quantum).round() / quantum
}

struct HitchLogWriter {
    file: std::io::BufWriter<std::fs::File>,
}

struct HitchLogState {
    writers: Vec<HitchLogWriter>,
    writes_since_flush: u32,
    last_flush: std::time::Instant,
}

impl HitchLogState {
    fn new() -> Self {
        let mut writers = Vec::new();
        for path in redraw_hitch_log_paths() {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if let Ok(file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                writers.push(HitchLogWriter {
                    file: std::io::BufWriter::with_capacity(16 * 1024, file),
                });
            }
        }

        Self {
            writers,
            writes_since_flush: 0,
            last_flush: std::time::Instant::now(),
        }
    }

    fn write_line(&mut self, msg: &str) {
        use std::io::Write as _;

        let mut i = 0;
        while i < self.writers.len() {
            let ok = self.writers[i].file.write_all(msg.as_bytes()).is_ok();
            if ok {
                i += 1;
            } else {
                self.writers.swap_remove(i);
            }
        }

        self.writes_since_flush = self.writes_since_flush.saturating_add(1);
        let should_flush =
            self.writes_since_flush >= 64 || self.last_flush.elapsed().as_millis() >= 250;
        if should_flush {
            for w in self.writers.iter_mut() {
                let _ = w.file.flush();
            }
            self.writes_since_flush = 0;
            self.last_flush = std::time::Instant::now();
        }
    }
}

fn write_redraw_hitch_log(line: &str) {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    let thread_id = format!("{:?}", std::thread::current().id());
    let msg = format!("[{ts}] [thread={thread_id}] {line}\n");

    static STATE: OnceLock<Mutex<HitchLogState>> = OnceLock::new();
    let state = STATE.get_or_init(|| Mutex::new(HitchLogState::new()));
    let mut state = state.lock().unwrap_or_else(|e| e.into_inner());
    state.write_line(&msg);
}

impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {
    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<winit::event::DeviceId>,
        event: DeviceEvent,
    ) {
        let dock_drag_pointer_id = self.dock_drag_pointer_id();
        if dock_drag_pointer_id.is_none() && self.dock_tearoff_follow.is_none() {
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

                #[cfg(target_os = "macos")]
                {
                    if !self.macos_refresh_cursor_screen_pos_from_nsevent() {
                        let _ = self.macos_bootstrap_cursor_transform_from_active_drag();
                    }
                    if !self.macos_refresh_cursor_screen_pos_from_nsevent() {
                        // Fallback: integrate pointer deltas. This is drift-prone on macOS, so we
                        // try hard to use `NSEvent::mouseLocation` + calibrated transforms first.
                        let Some(pos) = self.cursor_screen_pos else {
                            return;
                        };

                        if macos_cursor_trace_enabled() {
                            dock_tearoff_log(format_args!(
                                "[cursor-delta-fallback] prev=({:.1},{:.1}) delta=({:.1},{:.1})",
                                pos.x, pos.y, delta.0, delta.1
                            ));
                        }

                        self.cursor_screen_pos =
                            Some(PhysicalPosition::new(pos.x + delta.0, pos.y + delta.1));
                    }
                }

                #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
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
                // This fallback path is only for releases that occur outside all windows, where
                // winit may not emit `WindowEvent::MouseInput`.
                let Some(pointer_id) = dock_drag_pointer_id else {
                    return;
                };

                #[cfg(target_os = "windows")]
                if let Some(p) = win32::cursor_pos_physical() {
                    self.cursor_screen_pos = Some(p);
                }

                #[cfg(target_os = "macos")]
                {
                    if !self.macos_refresh_cursor_screen_pos_from_nsevent() {
                        let _ = self.macos_bootstrap_cursor_transform_from_active_drag();
                        let _ = self.macos_refresh_cursor_screen_pos_from_nsevent();
                    }
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
                    let Some(drag) = self.app.drag(pointer_id) else {
                        return;
                    };
                    if drag.kind != fret_app::DRAG_KIND_DOCK_PANEL {
                        return;
                    }
                    (drag.source_window, drag.current_window, drag.dragging)
                };
                dock_tearoff_log(format_args!(
                    "[device-up] pointer={:?} source={:?} current={:?} screen_pos={:?} dragging={}",
                    pointer_id, source_window, current_window, self.cursor_screen_pos, dragging
                ));

                #[cfg(target_os = "macos")]
                {
                    if self.saw_left_mouse_release_this_turn || macos_is_left_mouse_down() {
                        return;
                    }
                    if let Some(d) = self.app.drag_mut(pointer_id)
                        && d.kind == fret_app::DRAG_KIND_DOCK_PANEL
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
                if self
                    .app
                    .drag(pointer_id)
                    .is_some_and(|d| d.cross_window_hover)
                {
                    self.app.cancel_drag(pointer_id);
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
        self.app.with_global_mut(
            fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore::default,
            |store, _app| {
                store.record_can_create_surfaces();
            },
        );

        if self.context.is_some() {
            let Some(context) = self.context.as_ref() else {
                return;
            };

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

            for (app_window, state) in self.windows.iter_mut() {
                if state.surface.is_some() {
                    continue;
                }

                let surface = match context.create_surface(state.window.clone()) {
                    Ok(surface) => surface,
                    Err(e) => {
                        error!(window = ?app_window, error = ?e, "failed to recreate surface");
                        continue;
                    }
                };

                let size = state.window.surface_size();
                let surface_state = match SurfaceState::new_with_usage(
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
                            "failed to configure recreated surface"
                        );
                        continue;
                    }
                };

                state.surface = Some(surface_state);
                state.window.request_redraw();
            }

            self.drain_effects(event_loop);
            return;
        }

        let spec = self.config.main_window_spec();
        let window = match self.create_os_window(
            event_loop,
            spec,
            fret_runtime::WindowStyleRequest::default(),
            None,
        ) {
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

        if let Some(raw) = std::env::var_os("FRET_WGPU_BACKEND")
            && !raw.is_empty()
        {
            tracing::info!(requested = ?raw, "wgpu backend requested");
        }
        let info = context.adapter.get_info();
        tracing::info!(
            backend = ?info.backend,
            name = info.name,
            driver = info.driver,
            driver_info = info.driver_info,
            vendor = info.vendor,
            device = info.device,
            "wgpu adapter selected"
        );

        let renderer_caps = fret_render::RendererCapabilities::from_wgpu_context(&context);
        self.app
            .set_global::<fret_render::RendererCapabilities>(renderer_caps.clone());

        renderer.set_svg_raster_budget_bytes(self.config.svg_raster_budget_bytes);
        renderer.set_intermediate_budget_bytes(self.config.renderer_intermediate_budget_bytes);
        renderer.set_path_msaa_samples(self.config.path_msaa_samples);
        let _ = renderer.set_text_font_families(&self.config.text_font_families);
        let locale = self
            .app
            .global::<fret_runtime::fret_i18n::I18nService>()
            .and_then(|service| service.preferred_locales().first())
            .map(|locale| locale.to_string());
        let _ = renderer.set_text_locale(locale.as_deref());
        self.app
            .set_global::<fret_core::TextFontFamilyConfig>(self.config.text_font_families.clone());
        self.app
            .set_global::<fret_runtime::TextFontStackKey>(fret_runtime::TextFontStackKey(
                renderer.text_font_stack_key(),
            ));

        let entries = renderer
            .all_font_catalog_entries()
            .into_iter()
            .map(|e| fret_runtime::FontCatalogEntry {
                family: e.family,
                has_variable_axes: e.has_variable_axes,
                known_variable_axes: e.known_variable_axes,
                is_monospace_candidate: e.is_monospace_candidate,
            })
            .collect::<Vec<_>>();
        // Font catalog refresh trigger (ADR 0258): initial renderer availability (startup).
        let _ = fret_runtime::apply_font_catalog_update_with_metadata(
            &mut self.app,
            entries,
            fret_runtime::FontFamilyDefaultsPolicy::None,
        );

        self.context = Some(context);
        self.renderer = Some(renderer);
        self.renderer_caps = Some(renderer_caps);
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

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.app.with_global_mut(
            fret_runtime::RunnerSurfaceLifecycleDiagnosticsStore::default,
            |store, _app| {
                store.record_destroy_surfaces();
            },
        );

        for (_app_window, state) in self.windows.iter_mut() {
            state.surface = None;
            state.pending_surface_resize = None;
        }
        self.raf_windows.clear();
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
                #[cfg(windows)]
                RunnerUserEvent::WindowsMenuCommand { window, command } => {
                    self.app.push_effect(fret_app::Effect::Command {
                        window: Some(window),
                        command,
                    });
                }
                #[cfg(target_os = "macos")]
                RunnerUserEvent::MacosMenuCommand { window, command } => {
                    self.app
                        .push_effect(fret_app::Effect::Command { window, command });
                }
                #[cfg(target_os = "macos")]
                RunnerUserEvent::MacosMenuWillOpen => {
                    macos_menu::sync_command_gating_from_app(&self.app);
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
            ref ev @ WindowEvent::ModifiersChanged(..) => {
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.platform.handle_window_event(
                        state.window.scale_factor(),
                        ev,
                        &mut Vec::new(),
                    );
                }

                if self.dock_drag_pointer_id().is_some() {
                    self.route_internal_drag_hover_from_cursor();
                    self.drain_effects(event_loop);
                }
            }
            WindowEvent::ThemeChanged(_theme) => {
                let window_ref = self.windows.get(app_window).map(|s| s.window.clone());
                if let Some(window_ref) = window_ref {
                    if self
                        .update_window_environment_for_window_ref(app_window, window_ref.as_ref())
                    {
                        self.app.request_redraw(app_window);
                    }
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
                if let Some(state) = self.windows.get_mut(app_window) {
                    state.pending_surface_resize = Some(size);
                }
                self.app.request_redraw(app_window);
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

                if let Some(p) = screen_pos {
                    self.cursor_screen_pos = Some(p);
                    #[cfg(target_os = "macos")]
                    self.macos_calibrate_cursor_transform_from_window_sample(p, _scale_factor);
                }

                let _ = self.update_dock_tearoff_follow();

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
                    self.deliver_window_event_now(app_window, &evt);
                }

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
                    // Deliver the cursor-based drop on any left mouse release; this keeps docking
                    // robust even when the drag pointer id is not `PointerId(0)`.
                    self.route_internal_drag_drop_from_cursor();
                    if self.dock_tearoff_follow.is_some() {
                        self.stop_dock_tearoff_follow(Instant::now(), true);
                    }

                    // Cross-window drags are runner-routed (Enter/Over/Drop), so ensure the
                    // drag session cannot get "stuck" if no widget ends it.
                    if let Some(released) = left_up_pointer_id
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
                    self.deliver_window_event_now(app_window, &evt);
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
                    self.resize_surface(app_window, size.width, size.height);

                    // Keep delivering size/scale events for consistency with the existing runner
                    // behavior, but apply them once per frame so interactive resizes don't spam
                    // surface reconfigures and relayouts.
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

                    let prepare_started = hitch_config.map(|_| Instant::now());
                    let prepare_span = tracing::info_span!("fret.runner.prepare");
                    let _prepare_guard = prepare_span.enter();
                    // Apply any pending window-side state (IME/cursor) once per frame, similar to
                    // Dear ImGui's backend `prepare_frame` pattern.
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
                    if let Some(started) = prepare_started {
                        hitch_prepare_ms = Some(started.elapsed().as_millis() as u64);
                    }

                    let render_started = hitch_config.map(|_| Instant::now());
                    let render_span = tracing::info_span!(
                        "fret.runner.render",
                        bounds = ?bounds,
                        scale_factor = scale_factor,
                    );
                    let _render_guard = render_span.enter();
                    let render_text_diag_enabled = std::env::var_os("FRET_DIAG_DIR")
                        .is_some_and(|v| !v.is_empty())
                        || std::env::var_os("FRET_RENDER_TEXT_DEBUG")
                            .is_some_and(|v| !v.is_empty());
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
                    if let Some(started) = render_started {
                        hitch_render_ms = Some(started.elapsed().as_millis() as u64);
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
                        dirty |= state.platform.set_ime_allowed(snapshot.focus_is_text_input);
                        if snapshot.focus_is_text_input
                            && let Some(rect) = snapshot.ime_cursor_area
                        {
                            dirty |= state.platform.set_ime_cursor_area(rect);
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

                    let record_started = hitch_config.map(|_| Instant::now());
                    let record_span = tracing::info_span!(
                        "fret.runner.record",
                        scene_ops = state.scene.ops_len(),
                    );
                    let _record_guard = record_span.enter();
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
                    if let Some(started) = record_started {
                        hitch_record_ms = Some(started.elapsed().as_millis() as u64);
                    }

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

                    let present_started = hitch_config.map(|_| Instant::now());
                    let present_span = tracing::info_span!("fret.runner.present");
                    let _present_guard = present_span.enter();
                    let draw_result = (|| -> Result<(), fret_render::RenderError> {
                        let (frame, view) = surface.get_current_frame_view().map_err(|source| {
                            fret_render::RenderError::SurfaceAcquireFailed { source }
                        })?;

                        let screenshot_dir = self.diag_bundle_screenshots.poll_request_dir();

                        let render_scene_span = tracing::info_span!("fret.runner.render_scene");
                        let _render_scene_guard = render_scene_span.enter();
                        let ui_cmd = renderer.render_scene(
                            &context.device,
                            &context.queue,
                            fret_render::RenderSceneParams {
                                format: surface.format(),
                                target_view: &view,
                                scene: &state.scene,
                                clear: self.config.clear_color,
                                scale_factor,
                                viewport_size: surface.size(),
                            },
                        );
                        if render_text_diag_enabled {
                            self.app
                                .set_global(renderer.text_diagnostics_snapshot(self.frame_id));
                            self.app
                                .set_global(renderer.text_font_trace_snapshot(self.frame_id));
                        }

                        let diag_renderer_perf = std::env::var_os("FRET_DIAG_RENDERER_PERF")
                            .is_some_and(|v| !v.is_empty());
                        if diag_renderer_perf
                            && let Some(perf) = renderer.take_last_frame_perf_snapshot()
                        {
                            let tick_id = self.tick_id.0;
                            let frame_id = self.frame_id.0;
                            self.app.with_global_mut_untracked(
                                fret_render::RendererPerfFrameStore::default,
                                |store, _app| {
                                    store.record(app_window, tick_id, frame_id, perf);
                                },
                            );
                        }

                        let mut cmd_buffers = engine_frame.command_buffers;
                        cmd_buffers.push(ui_cmd);

                        #[cfg(feature = "diag-screenshots")]
                        let mut screenshot_inflight: Option<
                            diag_screenshots::InFlightCapture,
                        > = None;
                        #[cfg(feature = "diag-screenshots")]
                        if let Some(diag) = self.diag_screenshots.as_mut() {
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
                        if let Some(dir) = screenshot_dir
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
                        frame.present();

                        #[cfg(feature = "diag-screenshots")]
                        if let (Some(diag), Some(inflight)) =
                            (self.diag_screenshots.as_mut(), screenshot_inflight)
                        {
                            if let Err(err) = diag.finish_capture(&context.device, inflight) {
                                tracing::warn!(
                                    error = %err,
                                    window = ?app_window,
                                    "diag screenshot: capture failed"
                                );
                            }
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
                    })();
                    if let Some(started) = present_started {
                        hitch_present_ms = Some(started.elapsed().as_millis() as u64);
                    }

                    if capturing && let Some(r) = self.renderdoc.as_mut() {
                        r.end_capture();
                    }

                    if let Err(err) = draw_result {
                        match err {
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::Lost,
                            } => {
                                let size = state.window.surface_size();
                                surface.resize(&context.device, size.width, size.height);
                                state.window.request_redraw();
                                self.raf_windows.insert(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::Outdated,
                            } => {
                                let size = state.window.surface_size();
                                surface.resize(&context.device, size.width, size.height);
                                state.window.request_redraw();
                                self.raf_windows.insert(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::Timeout,
                            } => {
                                // Transient on some platforms (especially during startup / resize).
                                // Schedule a one-shot redraw so the window doesn't stay blank until
                                // the next user input arrives.
                                state.window.request_redraw();
                                self.raf_windows.insert(app_window);
                                return;
                            }
                            fret_render::RenderError::SurfaceAcquireFailed {
                                source: wgpu::SurfaceError::OutOfMemory,
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

                    self.frame_id.0 = self.frame_id.0.saturating_add(1);
                    self.app.set_frame_id(self.frame_id);
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

        self.tick_id.0 = self.tick_id.0.saturating_add(1);
        self.app.set_tick_id(self.tick_id);
        self.saw_left_mouse_release_this_turn = false;
        self.poll_window_environment_if_due(Instant::now());

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

                let mut insets_changed = false;
                self.app
                    .with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
                        if svc.safe_area_insets(app_window) != Some(safe_area_insets) {
                            svc.set_safe_area_insets(app_window, Some(safe_area_insets));
                            insets_changed = true;
                        }
                        if svc.occlusion_insets(app_window) != Some(occlusion_insets) {
                            svc.set_occlusion_insets(app_window, Some(occlusion_insets));
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

                let mut insets_changed = false;
                self.app
                    .with_global_mut(fret_core::WindowMetricsService::default, |svc, _app| {
                        if svc.safe_area_insets(app_window) != Some(safe_area_insets) {
                            svc.set_safe_area_insets(app_window, Some(safe_area_insets));
                            insets_changed = true;
                        }
                        if svc.occlusion_insets(app_window) != Some(occlusion_insets) {
                            svc.set_occlusion_insets(app_window, Some(occlusion_insets));
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

        if let Some(follow) = self.dock_tearoff_follow {
            // Stop follow even without pointer motion (e.g. Escape cancels the drag session).
            if self.dock_drag_pointer_id().is_none()
                || !self.is_left_mouse_down_for_window(follow.source_window)
            {
                self.stop_dock_tearoff_follow(Instant::now(), false);
            }
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

        if let Some(deadline) = self.dispatcher.next_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        if let Some(deadline) = self.next_pending_front_deadline() {
            next_deadline = Some(match next_deadline {
                Some(cur) => cur.min(deadline),
                None => deadline,
            });
        }

        #[cfg(feature = "hotpatch-subsecond")]
        if let Some(trigger) = self.hotpatch.as_ref() {
            if let Some(deadline) = trigger.next_poll_at() {
                next_deadline = Some(match next_deadline {
                    Some(cur) => cur.min(deadline),
                    None => deadline,
                });
            }
        }

        let drag_poll = self.dock_drag_pointer_id().is_some();
        let follow_poll = self.dock_tearoff_follow.is_some();
        let wants_poll = drag_poll || follow_poll;

        let wants_raf = !self.raf_windows.is_empty();
        if wants_raf {
            for app_window in self.raf_windows.drain() {
                if let Some(state) = self.windows.get(app_window) {
                    state.window.request_redraw();
                }
            }
        }

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
