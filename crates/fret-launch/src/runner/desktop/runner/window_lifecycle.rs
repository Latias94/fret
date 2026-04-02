use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn create_os_window(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        mut spec: WindowCreateSpec,
        style: WindowStyleRequest,
        _parent_window: Option<winit::raw_window_handle::RawWindowHandle>,
        caps: &PlatformCapabilities,
    ) -> Result<(Arc<dyn Window>, Option<accessibility::WinitAccessibility>), RunnerError> {
        spec.normalize_size_constraints();

        let accessibility_enabled = self.config.accessibility_enabled
            && std::env::var_os("FRET_A11Y_DISABLE").is_none_or(|v| v.is_empty());

        let mut attrs = winit::window::WindowAttributes::default()
            .with_title(spec.title)
            .with_surface_size(winit::dpi::LogicalSize::new(
                spec.size.width,
                spec.size.height,
            ))
            .with_visible(if accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(min_size) = spec.min_size {
            attrs = attrs.with_min_surface_size(winit::dpi::LogicalSize::new(
                min_size.width,
                min_size.height,
            ));
        }
        if let Some(max_size) = spec.max_size {
            attrs = attrs.with_max_surface_size(winit::dpi::LogicalSize::new(
                max_size.width,
                max_size.height,
            ));
        }
        if let Some(resize_increments) = spec.resize_increments {
            attrs = attrs.with_surface_resize_increments(winit::dpi::LogicalSize::new(
                resize_increments.width,
                resize_increments.height,
            ));
        }
        if let Some(resizable) = style.resizable {
            attrs = attrs.with_resizable(resizable);
        }
        if let Some(decorations) = style.decorations
            && matches!(decorations, fret_runtime::WindowDecorationsRequest::None)
        {
            attrs = attrs.with_decorations(false);
        }
        let effective_background_material = style.background_material.map(|m| {
            fret_runtime::runner_window_style_diagnostics::clamp_background_material_request(
                m, caps,
            )
        });

        let effective_surface_composited_alpha = if caps.ui.window_transparent {
            if let Some(transparent) = style.transparent {
                transparent
            } else {
                effective_background_material
                    .is_some_and(|m| m != fret_runtime::WindowBackgroundMaterialRequest::None)
            }
        } else {
            false
        };

        if caps.ui.window_transparent {
            // NOTE: `transparent` is a create-time property in winit; we may keep the window
            // composited for its lifetime even if the material is later set to None at runtime.
            attrs = attrs.with_transparent(effective_surface_composited_alpha);
        }
        if let Some(policy) = style.activation {
            let active = matches!(policy, ActivationPolicy::Activates);
            attrs = attrs.with_active(active);
        }
        if let Some(position) = spec.position {
            let position = match position {
                WindowPosition::Logical(pos) => winit::dpi::Position::Logical(
                    winit::dpi::LogicalPosition::new(pos.x as f64, pos.y as f64),
                ),
                WindowPosition::Physical(pos) => {
                    winit::dpi::Position::Physical(winit::dpi::PhysicalPosition::new(pos.x, pos.y))
                }
            };
            attrs = attrs.with_position(position);
        }
        #[cfg(windows)]
        {
            if let Some(taskbar) = style.taskbar {
                use winit::platform::windows::WindowAttributesWindows;

                let win = WindowAttributesWindows::default()
                    .with_skip_taskbar(matches!(taskbar, TaskbarVisibility::Hide));
                attrs = attrs.with_platform_attributes(Box::new(win));
            }
        }
        #[cfg(target_os = "macos")]
        if _parent_window.is_some() {
            // macOS tool/aux windows: best-effort parent/child relationship so DockFloating windows
            // follow the parent window's Space/fullscreen lifecycle.
            //
            // winit maps this to `NSWindow.addChildWindow_ordered(...)`.
            attrs = unsafe { attrs.with_parent_window(_parent_window) };
        }
        let window = Arc::<dyn Window>::from(
            event_loop
                .create_window(attrs)
                .map_err(|source| RunnerError::CreateWindowFailed { source })?,
        );

        macos_window_log(format_args!("[create] winit={:?}", window.id()));

        let accessibility = accessibility_enabled
            .then(|| accessibility::WinitAccessibility::new(event_loop, window.as_ref()));

        if accessibility_enabled && spec.visible {
            window.set_visible(true);
        }

        if let Some(level) = style.z_level {
            window.set_window_level(match level {
                WindowZLevel::Normal => WindowLevel::Normal,
                WindowZLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
            });
        }

        if effective_surface_composited_alpha
            && let Some(material) = effective_background_material
            && material != fret_runtime::WindowBackgroundMaterialRequest::None
        {
            let _ = super::window::set_window_background_material(window.as_ref(), material);
        }

        if let Some(hit_test) = style.hit_test.clone() {
            let (effective, _reason) =
                fret_runtime::RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request(
                    hit_test, caps,
                );
            let _ = super::window::set_window_hit_test(window.as_ref(), &effective);
        }
        if let Some(opacity) = style.opacity {
            let _ = super::window::set_window_opacity(window.as_ref(), opacity.as_f32());
        }

        Ok((window, accessibility))
    }

    pub(super) fn create_window_from_request(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        request: &CreateWindowRequest,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
        let mut spec = self
            .driver
            .window_create_spec(&mut self.app, request)
            .unwrap_or_else(|| self.config.default_window_spec());

        #[cfg(feature = "dev-state")]
        let dev_state_key: Option<String> = match &request.kind {
            CreateWindowKind::DockRestore { logical_window_id } => Some(logical_window_id.clone()),
            _ => None,
        };

        #[cfg(feature = "dev-state")]
        if self.dev_state.enabled()
            && let Some(key) = dev_state_key.as_deref()
        {
            self.dev_state.apply_window_spec(key, &mut spec);
            self.dev_state.sanitize_window_spec_position(
                key,
                &mut spec,
                event_loop
                    .available_monitors()
                    .filter_map(|m| Some((m.position()?, m.current_video_mode()?.size()))),
            );
        }

        spec.normalize_size_constraints();

        if spec.position.is_none() {
            // For dock tear-off, initially place near the cursor; we will refine the position
            // after the OS window exists using its own decoration offset (ImGui-style).
            if let CreateWindowKind::DockFloating { source_window, .. } = request.kind {
                if let Some(anchor) = request.anchor {
                    // Initial positioning is best-effort until the OS window exists, but it's
                    // worth approximating with the source window's decoration offset so Windows
                    // doesn't "jump" after creation under mixed DPI / non-client offsets.
                    spec.position = self.compute_window_position_from_cursor_grab_estimate(
                        anchor.window,
                        spec.size,
                        anchor.position,
                    );
                }
                if spec.position.is_none() {
                    spec.position = self.compute_window_position_from_cursor(source_window);
                }
            }

            if spec.position.is_none()
                && let Some(anchor) = request.anchor
            {
                spec.position = self.compute_window_position_from_anchor(anchor);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Avoid the "flash behind the source window" when tearing off a dock panel by
            // creating the new OS window hidden, then letting the deferred raise show it.
            if let CreateWindowKind::DockFloating { source_window, .. } = request.kind
                && !self.is_left_mouse_down_for_window(source_window)
            {
                spec.visible = false;
            }
        }

        #[cfg(target_os = "macos")]
        let parent_window = {
            use winit::raw_window_handle::HasWindowHandle as _;
            if !macos_dockfloating_parenting_enabled() {
                None
            } else {
                match request.kind {
                    CreateWindowKind::DockFloating { source_window, .. } => self
                        .windows
                        .get(source_window)
                        .and_then(|w| w.window.window_handle().ok())
                        .map(|h| h.as_raw()),
                    _ => None,
                }
            }
        };
        #[cfg(not(target_os = "macos"))]
        let parent_window = None;

        let style = request.style.clone();
        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let (window, accessibility) =
            self.create_os_window(event_loop, spec, style.clone(), parent_window, &caps)?;
        let surface = {
            let Some(context) = self.context.as_ref() else {
                return Err(RunnerError::WgpuNotInitialized);
            };
            context.create_surface(window.clone())?
        };
        let new_window = self.insert_window(window, accessibility, Some(surface), style.clone())?;
        self.app.with_global_mut(
            fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_open(new_window, style, &caps);
            },
        );

        #[cfg(feature = "dev-state")]
        if self.dev_state.enabled()
            && let Some(key) = dev_state_key
        {
            self.dev_state.register_window_key(new_window, key);
        }

        Ok(new_window)
    }

    pub(super) fn enqueue_window_front(
        &mut self,
        window: fret_core::AppWindowId,
        source_window: Option<fret_core::AppWindowId>,
        panel: Option<fret_core::PanelKey>,
        now: Instant,
    ) {
        macos_window_log(format_args!(
            "[enqueue-front] target={:?} source={:?} now={:?}",
            window, source_window, now
        ));

        // macOS may ignore focus changes during an active interaction in the source window.
        // Retry a few times over subsequent event-loop turns (and stop once the window reports
        // `Focused(true)`).
        self.windows_pending_front.insert(
            window,
            PendingFrontRequest {
                source_window,
                panel,
                created_at: now,
                // Defer the first raise to `about_to_wait` (Godot uses `call_deferred`); this
                // avoids fighting the platform while a tracked interaction is still active.
                next_attempt_at: now,
                attempts_left: 10,
            },
        );
    }

    pub(super) fn process_pending_front_requests(&mut self, now: Instant) -> bool {
        if self.windows_pending_front.is_empty() {
            return false;
        }

        let pending = std::mem::take(&mut self.windows_pending_front);
        let mut kept: HashMap<fret_core::AppWindowId, PendingFrontRequest> = HashMap::new();
        let mut did_work = false;

        for (window, mut req) in pending {
            let Some(state) = self.windows.get(window) else {
                continue;
            };

            if state.is_focused && req.attempts_left > 2 {
                // Even after winit reports the window focused, the window ordering can still lag
                // behind when the float was initiated from a tracked menu / drag sequence.
                // Keep a couple more retries to ensure it actually surfaces.
                req.attempts_left = 2;
            }

            if req.attempts_left == 0 {
                macos_window_log(format_args!(
                    "[front-done] target={:?} panel={:?} focused={} age_ms={} now={:?}",
                    window,
                    req.panel.as_ref().map(|p| &p.kind.0),
                    state.is_focused,
                    now.saturating_duration_since(req.created_at).as_millis(),
                    now,
                ));
                continue;
            }

            if now >= req.next_attempt_at {
                macos_window_log(format_args!(
                    "[front-try] target={:?} panel={:?} source={:?} focused={} attempts_left={} age_ms={} now={:?}",
                    window,
                    req.panel.as_ref().map(|p| &p.kind.0),
                    req.source_window,
                    state.is_focused,
                    req.attempts_left,
                    now.saturating_duration_since(req.created_at).as_millis(),
                    now,
                ));
                let sender = req
                    .source_window
                    .and_then(|id| self.windows.get(id))
                    .map(|w| w.window.as_ref());
                let _ = bring_window_to_front(state.window.as_ref(), sender);
                state.window.request_redraw();
                req.attempts_left = req.attempts_left.saturating_sub(1);
                req.next_attempt_at = now + Duration::from_millis(60);
                did_work = true;
            }

            kept.insert(window, req);
        }

        self.windows_pending_front = kept;
        did_work
    }

    pub(super) fn next_pending_front_deadline(&self) -> Option<Instant> {
        self.windows_pending_front
            .values()
            .filter(|r| r.attempts_left > 0)
            .map(|r| r.next_attempt_at)
            .min()
    }

    pub(super) fn insert_window(
        &mut self,
        window: Arc<dyn Window>,
        accessibility: Option<accessibility::WinitAccessibility>,
        surface: Option<wgpu::Surface<'static>>,
        style: WindowStyleRequest,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
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
                want_surface_composited_alpha_for_style(style, &caps);
            configure_surface_alpha_mode_for_composited_window(
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
        // install the `WindowId` -> `AppWindowId` mapping *before* requesting the redraw. Otherwise, the first
        // redraw can be dropped and the window may appear blank until another event arrives.
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

    pub(super) fn close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, true)
    }

    pub(super) fn force_close_window(&mut self, window: fret_core::AppWindowId) -> bool {
        self.close_window_impl(window, false)
    }

    fn close_window_impl(
        &mut self,
        window: fret_core::AppWindowId,
        check_before_close: bool,
    ) -> bool {
        if !self.windows.contains_key(window) {
            return false;
        }

        if check_before_close {
            let should_close = self.driver.before_close_window(&mut self.app, window);
            if !should_close {
                return false;
            }
        }

        #[cfg(feature = "dev-state")]
        if check_before_close && self.dev_state.enabled() {
            let alive: std::collections::HashSet<fret_core::AppWindowId> =
                self.windows.keys().collect();
            self.dev_state
                .sync_window_keys_from_app(&self.app, |window| alive.contains(&window));

            let key = self.dev_state.window_key(window).map(ToString::to_string);
            if let Some(key) = key
                && let Some(state) = self.windows.get(window)
            {
                let physical = state.window.surface_size();
                let logical: winit::dpi::LogicalSize<f64> =
                    physical.to_logical(state.window.scale_factor());
                let position = state.window.outer_position().ok();
                self.dev_state
                    .observe_window_geometry_now(&key, logical, position);
            }

            self.dev_state.export_and_flush_now(&mut self.app);
        }

        if self
            .dock_tearoff_follow
            .is_some_and(|f| f.window == window || f.source_window == window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }
        self.dock_floating_windows.remove(&window);

        if self.internal_drag_hover_window == Some(window) {
            self.internal_drag_hover_window = None;
            self.internal_drag_hover_pos = None;
            self.internal_drag_pointer_id = None;
        }

        {
            use fret_runtime::DragHost as _;
            use std::collections::HashSet;

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.source_window == window
            }) {
                visited.insert(pointer_id);
                self.app.cancel_drag(pointer_id);
            }

            let mut visited: HashSet<fret_core::PointerId> = HashSet::new();
            while let Some(pointer_id) = self.app.find_drag_pointer_id(|d| {
                !visited.contains(&d.pointer_id) && d.current_window == window
            }) {
                visited.insert(pointer_id);
                if let Some(drag) = self.app.drag_mut(pointer_id) {
                    drag.current_window = drag.source_window;
                }
            }
        }

        self.webviews.close_window(&mut self.app, window);

        let Some(state) = self.windows.remove(window) else {
            return false;
        };
        #[cfg(feature = "dev-state")]
        self.dev_state.unregister_window(window);
        self.windows_z_order.retain(|w| *w != window);
        #[cfg(windows)]
        windows_menu::unregister_window(state.window.as_ref());
        #[cfg(target_os = "macos")]
        macos_menu::unregister_window(state.window.as_ref());
        #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
        macos_hit_test::unregister_window(state.window.as_ref());
        self.window_registry.remove(state.window.id());
        self.app.with_global_mut(
            fret_runtime::RunnerWindowLifecycleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_close(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::RunnerWindowStyleDiagnosticsStore::default,
            |svc, _app| {
                svc.record_window_close(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerPresentDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerFrameDriveDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::WindowRedrawRequestDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::WindowGlobalChangeDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );
        self.app.with_global_mut_untracked(
            fret_runtime::RunnerSurfaceConfigDiagnosticsStore::default,
            |svc, _app| {
                svc.clear_window(window);
            },
        );

        self.app.with_global_mut(
            fret_runtime::WindowInputContextService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandActionAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandAvailabilityService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandEnabledService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowCommandGatingService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app.with_global_mut(
            fret_runtime::WindowTextInputSnapshotService::default,
            |svc, _app| {
                svc.remove_window(window);
            },
        );
        self.app
            .with_global_mut(WindowMetricsService::default, |svc, _app| {
                svc.remove(window);
            });
        if Some(window) == self.main_window {
            self.main_window = None;
        }

        true
    }
}

fn want_surface_composited_alpha_for_style(
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
        surface.surface.configure(device, &surface.config);
    }
}
