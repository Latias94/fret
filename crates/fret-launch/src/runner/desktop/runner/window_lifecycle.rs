use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn create_os_window(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        spec: WindowCreateSpec,
        style: WindowStyleRequest,
        _parent_window: Option<winit::raw_window_handle::RawWindowHandle>,
    ) -> Result<(Arc<dyn Window>, Option<accessibility::WinitAccessibility>), RunnerError> {
        let accessibility_enabled = self.config.accessibility_enabled
            && std::env::var_os("FRET_A11Y_DISABLE").is_none_or(|v| v.is_empty());

        let mut attrs = winit::window::WindowAttributes::default()
            .with_title(spec.title)
            .with_surface_size(spec.size)
            .with_visible(if accessibility_enabled {
                false
            } else {
                spec.visible
            });
        if let Some(policy) = style.activation {
            let active = matches!(policy, ActivationPolicy::Activates);
            attrs = attrs.with_active(active);
        }
        if let Some(position) = spec.position {
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

        if let Some(mouse) = style.mouse {
            let passthrough = matches!(mouse, fret_runtime::MousePolicy::Passthrough);
            let _ = super::window::set_window_mouse_passthrough(window.as_ref(), passthrough);
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

        let (window, accessibility) =
            self.create_os_window(event_loop, spec, request.style, parent_window)?;
        let surface = {
            let Some(context) = self.context.as_ref() else {
                return Err(RunnerError::WgpuNotInitialized);
            };
            context.create_surface(window.clone())?
        };
        let new_window = self.insert_window(window, accessibility, Some(surface))?;

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
            Some(SurfaceState::new_with_usage(
                &context.adapter,
                &context.device,
                surface,
                size.width,
                size.height,
                surface_usage,
            )?)
        } else {
            None
        };

        let id = self.windows.insert_with_key(|id| {
            let user = self.driver.create_window_state(&mut self.app, id);
            WindowRuntime {
                window,
                accessibility,
                last_accessibility_snapshot: None,
                surface,
                scene: Scene::default(),
                platform: fret_runner_winit::WinitPlatform {
                    wheel: fret_runner_winit::WheelConfig {
                        line_delta_px: self.config.wheel_line_delta_px,
                        pixel_delta_scale: self.config.wheel_pixel_delta_scale,
                    },
                    ..Default::default()
                },
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
        if let Some(state) = self.windows.get(id) {
            state.window.request_redraw();
            // `request_redraw()` alone may not wake the event loop on some platforms; schedule a
            // one-shot RAF so the initial frame presents without requiring any user input.
            self.raf_windows.insert(id);
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

        #[cfg(feature = "webview-wry")]
        {
            let events = self.webviews_wry.destroy_all_for_window(window);
            let mut ids = Vec::new();
            for ev in &events {
                if let fret_webview::WebViewEvent::Destroyed { id } = ev {
                    ids.push(*id);
                }
            }

            if !events.is_empty() {
                fret_webview::webview_push_events(&mut self.app, events);
            }

            // Clear any registered surfaces (even if no backend instance was created yet).
            let removed = fret_webview::webview_remove_surfaces_for_window(&mut self.app, window);
            ids.extend(removed.into_iter().map(|s| s.id));
            ids.sort_by_key(|id| id.0);
            ids.dedup();

            // Drop any queued requests for this window/ids to avoid requeue loops (e.g. a `Create`
            // request for a closed window).
            if !ids.is_empty() {
                let _ = fret_webview::webview_drop_requests_for_window_close(
                    &mut self.app,
                    window,
                    &ids,
                );
            }
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
        self.window_registry.remove(state.window.id());

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
