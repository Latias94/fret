use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        event: DeviceEvent,
    ) {
        let internal_drag_pointer_id = self.internal_drag_routing_pointer_id();
        let dock_drag_pointer_id = self.dock_drag_pointer_id();
        if internal_drag_pointer_id.is_none() && self.dock_tearoff_follow.is_none() {
            return;
        }

        let isolate_pointer_input = self.diag_pointer_input_isolation_active();

        match event {
            DeviceEvent::PointerMotion { delta } => {
                if isolate_pointer_input {
                    return;
                }
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
                if dock_drag_pointer_id.is_some() || self.dock_tearoff_follow.is_some() {
                    let _ = self.update_dock_tearoff_follow();
                    self.sync_dock_drag_pointer_capture();
                }
                self.drain_effects(event_loop);
            }
            DeviceEvent::Button {
                state: ElementState::Released,
                ..
            } => {
                if isolate_pointer_input {
                    return;
                }
                // This fallback path is only for releases that occur outside all windows, where
                // winit may not emit `WindowEvent::MouseInput`.
                let Some(pointer_id) = internal_drag_pointer_id else {
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
                if let Some(pos) = self.cursor_screen_pos {
                    let caps = self
                        .app
                        .global::<PlatformCapabilities>()
                        .cloned()
                        .unwrap_or_default();
                    let reliable_window_under_cursor = caps.ui.window_hover_detection
                        == fret_runtime::WindowHoverDetectionQuality::Reliable;
                    if reliable_window_under_cursor
                        && self
                            .window_under_cursor_platform(pos, None)
                            .window
                            .is_some()
                    {
                        return;
                    }
                }

                // Releasing the mouse button outside any window may not deliver a
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

                if self.saw_left_mouse_release_this_turn
                    || !self.is_left_mouse_down_for_window(source_window)
                {
                    return;
                }

                #[cfg(target_os = "macos")]
                if macos_is_left_mouse_down() {
                    return;
                }

                // We didn't observe a window-scoped mouse release, so clear the runner's cached
                // button state to avoid getting stuck in a "mouse down" state.
                self.left_mouse_down = false;
                for state in self.windows.values_mut() {
                    state.platform.input.pressed_buttons.left = false;
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
}
