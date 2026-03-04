use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn dock_drag_pointer_id(&self) -> Option<fret_core::PointerId> {
        use fret_runtime::DragHost as _;
        self.app.find_drag_pointer_id(|d| {
            d.cross_window_hover
                && (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                    || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
        })
    }

    pub(super) fn sync_dock_drag_pointer_capture(&mut self) {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            self.dock_drag_pointer_capture = None;
            return;
        };
        let Some(drag) = self.app.drag(pointer_id) else {
            self.dock_drag_pointer_capture = None;
            return;
        };

        let desired_window = drag.source_window;
        let Some((captured_pointer, captured_window)) = self.dock_drag_pointer_capture else {
            self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
            return;
        };

        if captured_pointer != pointer_id {
            self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
            return;
        }

        if captured_window == desired_window {
            return;
        }

        // When docking tear-off migrates a drag session to a new window, the original window can
        // remain stuck in a "pointer down" state because the eventual `PointerUp` is delivered to
        // the new window. Once the drag's `source_window` changes, the old window is no longer
        // considered part of the drag (source/current), so we can safely send it a cancel to
        // release pending pointer capture/press state without terminating the active drag.
        self.deliver_dock_drag_pointer_cancel(captured_window, pointer_id);

        self.dock_drag_pointer_capture = Some((pointer_id, desired_window));
    }

    fn deliver_dock_drag_pointer_cancel(
        &mut self,
        window: fret_core::AppWindowId,
        pointer_id: fret_core::PointerId,
    ) {
        let modifiers = self
            .windows
            .get(window)
            .map(|w| w.platform.input.modifiers)
            .unwrap_or_default();
        let position = self
            .cursor_screen_pos
            .and_then(|screen| self.local_pos_for_window(window, screen));
        let buttons = fret_core::MouseButtons {
            left: self.left_mouse_down,
            right: false,
            middle: false,
        };
        self.deliver_window_event_now(
            window,
            &Event::PointerCancel(fret_core::PointerCancelEvent {
                pointer_id,
                position,
                buttons,
                modifiers,
                pointer_type: fret_core::PointerType::Mouse,
                reason: fret_core::PointerCancelReason::LeftWindow,
            }),
        );
    }

    #[cfg(target_os = "macos")]
    pub(super) fn maybe_finish_dock_drag_released_outside(&mut self) -> bool {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        // Scripted diagnostics inject pointer events without a real OS mouse button state. When
        // pointer input isolation is active, avoid OS polling heuristics to terminate the drag;
        // scripts will deliver an explicit `PointerUp`.
        if self.diag_pointer_input_isolation_active() {
            return false;
        }

        let (source_window, current_window, dragging) = {
            let Some(drag) = self.app.drag(pointer_id) else {
                return false;
            };
            if !drag.cross_window_hover
                || (drag.kind != fret_runtime::DRAG_KIND_DOCK_PANEL
                    && drag.kind != fret_runtime::DRAG_KIND_DOCK_TABS)
                || macos_is_left_mouse_down()
                || self.saw_left_mouse_release_this_turn
            {
                return false;
            }
            (drag.source_window, drag.current_window, drag.dragging)
        };

        dock_tearoff_log(format_args!(
            "[poll-up] pointer={:?} source={:?} current={:?} screen_pos={:?} dragging={}",
            pointer_id, source_window, current_window, self.cursor_screen_pos, dragging
        ));

        // If the mouse was released outside any window, winit may not deliver a `MouseInput`
        // event to any window. Use the regular cursor-based drop routing so docking back into an
        // existing window still works (ImGui-style).
        if let Some(d) = self.app.drag_mut(pointer_id)
            && (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
        {
            d.dragging = true;
        }

        self.route_internal_drag_drop_from_cursor();
        dock_tearoff_log(format_args!(
            "[poll-drop] dispatched target={:?}",
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

        true
    }

    #[cfg(target_os = "windows")]
    pub(super) fn maybe_finish_dock_drag_released_outside_windows(&mut self) -> bool {
        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        // Scripted diagnostics inject pointer events without a real OS mouse button state. When
        // pointer input isolation is active, avoid OS polling heuristics to terminate the drag;
        // scripts will deliver an explicit `PointerUp`.
        if self.diag_pointer_input_isolation_active() {
            diag_dock_drag_trace(format_args!(
                "[poll-up-win32-skip] tick={} pointer={:?} reason=diag_pointer_input_isolation_active",
                self.tick_id.0, pointer_id
            ));
            return false;
        }

        let os_left_down = win32::is_left_mouse_down();
        let saw_release_this_turn = self.saw_left_mouse_release_this_turn;
        let (source_window, current_window, dragging) = {
            let Some(drag) = self.app.drag(pointer_id) else {
                diag_dock_drag_trace(format_args!(
                    "[poll-up-win32-skip] tick={} pointer={:?} reason=no_drag",
                    self.tick_id.0, pointer_id
                ));
                return false;
            };
            if !drag.cross_window_hover
                || (drag.kind != fret_runtime::DRAG_KIND_DOCK_PANEL
                    && drag.kind != fret_runtime::DRAG_KIND_DOCK_TABS)
                // Avoid mis-triggering this poll-up fallback when diagnostics scripts inject pointer
                // events (bypassing OS button state): only run when the runner believes the left
                // button is currently down for the drag's source window.
                || !self.is_left_mouse_down_for_window(drag.source_window)
                || os_left_down
                || saw_release_this_turn
            {
                diag_dock_drag_trace(format_args!(
                    "[poll-up-win32-skip] tick={} pointer={:?} cross_window_hover={} kind={:?} runner_left_down={} os_left_down={} saw_release_this_turn={}",
                    self.tick_id.0,
                    pointer_id,
                    drag.cross_window_hover,
                    drag.kind,
                    self.is_left_mouse_down_for_window(drag.source_window),
                    os_left_down,
                    saw_release_this_turn,
                ));
                return false;
            }
            (drag.source_window, drag.current_window, drag.dragging)
        };

        // Prefer the diagnostics cursor override if present; scripted runs cannot reliably
        // control OS cursor position, so clobbering `cursor_screen_pos` here can make poll-up
        // drop routing non-deterministic.
        if (self.diag_cursor_screen_pos_override.is_none() || self.cursor_screen_pos.is_none())
            && let Some(p) = win32::cursor_pos_physical()
        {
            self.cursor_screen_pos = Some(p);
        }

        diag_dock_drag_trace(format_args!(
            "[poll-up-win32] tick={} pointer={:?} source={:?} current={:?} screen_pos={:?} dragging={}",
            self.tick_id.0,
            pointer_id,
            source_window,
            current_window,
            self.cursor_screen_pos,
            dragging
        ));

        // If the release was not delivered as a window-scoped `MouseInput`, finish the drag using
        // the cursor-based drop routing (ImGui-style).
        if let Some(d) = self.app.drag_mut(pointer_id)
            && (d.kind == fret_runtime::DRAG_KIND_DOCK_PANEL
                || d.kind == fret_runtime::DRAG_KIND_DOCK_TABS)
        {
            d.dragging = true;
        }

        self.route_internal_drag_drop_from_cursor();
        diag_dock_drag_trace(format_args!(
            "[poll-drop-win32] tick={} pointer={:?} ok=true",
            self.tick_id.0, pointer_id
        ));

        if self
            .app
            .drag(pointer_id)
            .is_some_and(|d| d.cross_window_hover)
        {
            self.app.cancel_drag(pointer_id);
            let _ = self.clear_internal_drag_hover_if_needed();
        }

        if self.dock_tearoff_follow.is_some() {
            self.left_mouse_down = false;
            for state in self.windows.values_mut() {
                state.platform.input.pressed_buttons.left = false;
            }
            self.stop_dock_tearoff_follow(Instant::now(), false);
        }

        true
    }

    pub(super) fn update_dock_tearoff_follow(&mut self) -> bool {
        let pointer_id = self.dock_drag_pointer_id();
        if self.dock_tearoff_follow.is_some() && pointer_id.is_none() {
            // If the dock drag session was canceled (e.g. Escape), ensure we do not keep moving a
            // dock tear-off window indefinitely.
            self.stop_dock_tearoff_follow(Instant::now(), false);
            return true;
        }

        // Scripted diagnostics drive cursor position via overrides. Tear-off follow intentionally
        // moves OS windows to keep a real cursor "inside" the moving window; during scripted runs
        // this causes the runner to chase synthetic cursor updates and can prevent docking-back
        // gestures from ever reaching a stable overlap/hover state.
        //
        // When a recent diagnostics cursor/button override was observed, stop following and let
        // cursor-based hover/drop routing drive the interaction deterministically.
        if self.diag_pointer_input_isolation_active() && self.dock_tearoff_follow.is_some() {
            self.stop_dock_tearoff_follow(Instant::now(), false);
            return true;
        }

        if self.dock_tearoff_follow.is_none()
            && let Some(pointer_id) = pointer_id
            && let Some(drag) = self.app.drag(pointer_id)
        {
            let grab_offset = drag
                .cursor_grab_offset
                .unwrap_or(Point::new(Px(40.0), Px(20.0)));
            let settings = self
                .app
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            let want_transparent_payload = settings.transparent_payload_during_follow
                || std::env::var_os("FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD").is_some();

            let follow_window = match drag.kind {
                // If transparent payload is explicitly enabled, force follow so hover selection
                // can "peek behind" the moving window during overlap diagnostics.
                fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                    if want_transparent_payload =>
                {
                    Some(drag.source_window)
                }
                _ => drag.follow_window,
            };

            if let Some(window) = follow_window {
                self.dock_tearoff_follow = Some(super::DockTearoffFollow {
                    window,
                    source_window: drag.source_window,
                    grab_offset,
                    manual_follow: true,
                    last_outer_pos: None,
                    transparent_payload_applied: false,
                    mouse_passthrough_applied: false,
                    always_on_top_applied: false,
                });
            }
        }

        let (window, grab_offset, manual_follow, last_outer_pos, transparent_payload_applied) =
            match self.dock_tearoff_follow {
                Some(follow) => (
                    follow.window,
                    follow.grab_offset,
                    follow.manual_follow,
                    follow.last_outer_pos,
                    follow.transparent_payload_applied,
                ),
                None => return false,
            };

        if !manual_follow {
            return false;
        }

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        if caps.ui.window_set_outer_position
            != fret_runtime::WindowSetOuterPositionQuality::Reliable
        {
            return false;
        }

        if self.windows.get(window).is_none() {
            self.dock_tearoff_follow = None;
            return false;
        }

        // Optional ImGui-style "transparent payload" behavior while following the cursor:
        // - make the dock-floating window semi-transparent
        // - (best-effort) make the dock-floating window ignore mouse events (click-through)
        //
        // This is conservatively disabled by default (see `DockingInteractionSettings`), and can
        // be forced on via env var for quick experimentation.
        let settings = self
            .app
            .global::<fret_runtime::DockingInteractionSettings>()
            .copied()
            .unwrap_or_default();
        let want_transparent_payload = settings.transparent_payload_during_follow
            || std::env::var_os("FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD").is_some();

        if want_transparent_payload != transparent_payload_applied {
            let opacity = if want_transparent_payload {
                fret_runtime::WindowOpacity::from_f32(settings.transparent_payload_alpha)
            } else {
                fret_runtime::WindowOpacity(255)
            };
            self.app.push_effect(fret_app::Effect::Window(
                fret_app::WindowRequest::SetStyle {
                    window,
                    style: fret_runtime::WindowStyleRequest {
                        hit_test: Some(if want_transparent_payload {
                            fret_runtime::WindowHitTestRequestV1::PassthroughAll
                        } else {
                            fret_runtime::WindowHitTestRequestV1::Normal
                        }),
                        opacity: Some(opacity),
                        ..Default::default()
                    },
                },
            ));
            if let Some(follow) = self.dock_tearoff_follow.as_mut() {
                follow.transparent_payload_applied = want_transparent_payload;
            }
            if let Some(pointer_id) = pointer_id
                && let Some(drag) = self.app.drag_mut(pointer_id)
            {
                drag.transparent_payload_applied = want_transparent_payload;
                if !want_transparent_payload {
                    drag.transparent_payload_hit_test_passthrough_applied = false;
                }
            }
        }

        let Some(pos) = self.compute_window_outer_position_from_cursor_grab(window, grab_offset)
        else {
            return false;
        };

        let next_phys = {
            let Some(state) = self.windows.get(window) else {
                self.dock_tearoff_follow = None;
                return false;
            };
            let scale_factor = state.window.scale_factor();
            match pos {
                WindowPosition::Physical(p) => winit::dpi::PhysicalPosition::new(p.x, p.y),
                WindowPosition::Logical(p) => {
                    winit::dpi::LogicalPosition::new(p.x as f64, p.y as f64)
                        .to_physical::<i32>(scale_factor)
                }
            }
        };

        // Avoid spamming redundant position updates (helps reduce stutter on high-frequency
        // input devices).
        if last_outer_pos.is_some_and(|prev| prev == next_phys) {
            return false;
        }

        if let Some(state) = self.windows.get(window) {
            let pos = match pos {
                WindowPosition::Logical(p) => winit::dpi::Position::Logical(
                    winit::dpi::LogicalPosition::new(p.x as f64, p.y as f64),
                ),
                WindowPosition::Physical(p) => {
                    winit::dpi::Position::Physical(winit::dpi::PhysicalPosition::new(p.x, p.y))
                }
            };
            state.window.set_outer_position(pos);
        }

        dock_tearoff_log(format_args!(
            "[follow-move] window={:?} cursor={:?} outer_pos={:?}",
            window, self.cursor_screen_pos, next_phys
        ));

        if let Some(follow) = self.dock_tearoff_follow.as_mut() {
            follow.last_outer_pos = Some(next_phys);
        }

        true
    }

    pub(super) fn stop_dock_tearoff_follow(&mut self, _now: Instant, _raise_on_macos: bool) {
        let Some(follow) = self.dock_tearoff_follow.take() else {
            return;
        };

        dock_tearoff_log(format_args!(
            "[follow-stop] window={:?} source={:?} cursor={:?} raise_on_macos={}",
            follow.window, follow.source_window, self.cursor_screen_pos, _raise_on_macos
        ));

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();

        if follow.transparent_payload_applied {
            self.app.push_effect(fret_app::Effect::Window(
                fret_app::WindowRequest::SetStyle {
                    window: follow.window,
                    style: fret_runtime::WindowStyleRequest {
                        hit_test: Some(fret_runtime::WindowHitTestRequestV1::Normal),
                        opacity: Some(fret_runtime::WindowOpacity(255)),
                        ..Default::default()
                    },
                },
            ));
        }
        if let Some(pointer_id) = self.dock_drag_pointer_id()
            && let Some(drag) = self.app.drag_mut(pointer_id)
        {
            drag.transparent_payload_applied = false;
            drag.transparent_payload_hit_test_passthrough_applied = false;
        }

        if let Some(state) = self.windows.get(follow.window) {
            if caps.ui.window_z_level != fret_runtime::WindowZLevelQuality::None
                && follow.always_on_top_applied
            {
                self.app.push_effect(fret_app::Effect::Window(
                    fret_app::WindowRequest::SetStyle {
                        window: follow.window,
                        style: fret_runtime::WindowStyleRequest {
                            z_level: Some(fret_runtime::WindowZLevel::Normal),
                            ..Default::default()
                        },
                    },
                ));
            }
            if caps.ui.window_set_outer_position
                == fret_runtime::WindowSetOuterPositionQuality::Reliable
                && let Some(pos) =
                    self.settle_window_outer_position(state.window.as_ref(), self.cursor_screen_pos)
            {
                state.window.set_outer_position(Position::Physical(pos));
            }
        }

        #[cfg(target_os = "macos")]
        if _raise_on_macos {
            self.enqueue_window_front(follow.window, Some(follow.source_window), None, _now);
        }
    }
}
