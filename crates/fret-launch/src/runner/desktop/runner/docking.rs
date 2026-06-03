use super::*;

mod follow;

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
}
