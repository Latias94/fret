use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn insert_window(
        &mut self,
        window: Arc<dyn Window>,
        accessibility: Option<accessibility::WinitAccessibility>,
        surface: wgpu::Surface<'static>,
    ) -> Result<fret_core::AppWindowId, RunnerError> {
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
        let surface = SurfaceState::new_with_usage(
            &context.adapter,
            &context.device,
            surface,
            size.width,
            size.height,
            surface_usage,
        )?;

        let id = self.windows.insert_with_key(|id| {
            let user = self.driver.create_window_state(&mut self.app, id);
            WindowRuntime {
                window,
                accessibility,
                last_accessibility_snapshot: None,
                surface: Some(surface),
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
        if let Some(window_ref) = window_ref {
            if self.update_window_environment_for_window_ref(id, window_ref.as_ref()) {
                self.app.request_redraw(id);
            }
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

        if self
            .dock_tearoff_follow
            .is_some_and(|f| f.window == window || f.source_window == window)
        {
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }

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
