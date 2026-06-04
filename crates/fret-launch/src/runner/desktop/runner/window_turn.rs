use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_about_to_wait_window_platform_and_accessibility(&mut self) {
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
    }
}
