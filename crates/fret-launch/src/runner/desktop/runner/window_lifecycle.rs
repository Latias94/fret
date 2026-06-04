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
        if let Some(resizable) = style.resizable
            && caps.ui.window_resizable
        {
            attrs = attrs.with_resizable(resizable);
        }
        if let Some(decorations) = style.decorations
            && caps.ui.window_decorations
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
        if let Some(policy) = style.activation
            && (policy == ActivationPolicy::Activates || caps.ui.window_non_activating)
        {
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
            if let Some(taskbar) = style.taskbar
                && (taskbar == TaskbarVisibility::Show || caps.ui.window_skip_taskbar)
            {
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

        if let Some(level) = style.z_level
            && (level == WindowZLevel::Normal
                || caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None)
        {
            window.set_window_level(match level {
                WindowZLevel::Normal => WindowLevel::Normal,
                WindowZLevel::AlwaysOnTop => WindowLevel::AlwaysOnTop,
            });
        }

        if effective_surface_composited_alpha
            && let Some(material) = effective_background_material
            && material != fret_runtime::WindowBackgroundMaterialRequest::None
        {
            let _ =
                super::window_platform::set_window_background_material(window.as_ref(), material);
        }

        if let Some(hit_test) = style.hit_test.clone() {
            let (effective, _reason) =
                fret_runtime::RunnerWindowStyleDiagnosticsStore::clamp_hit_test_request(
                    hit_test, caps,
                );
            let _ = super::window_platform::set_window_hit_test(window.as_ref(), &effective);
        }
        if let Some(opacity) = style.opacity
            && caps.ui.window_opacity
        {
            let _ = super::window_platform::set_window_opacity(window.as_ref(), opacity.as_f32());
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

        self.refresh_runner_monitor_topology_diagnostics(event_loop);

        Ok(new_window)
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
}
