use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
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
}
