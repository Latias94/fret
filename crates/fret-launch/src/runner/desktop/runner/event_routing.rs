use super::*;

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn deliver_window_event_now(
        &mut self,
        window: fret_core::AppWindowId,
        event: &Event,
    ) {
        if self.maybe_handle_hotpatch_event(window, event) {
            return;
        }
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        fret_runtime::apply_window_metrics_event(&mut self.app, window, event);
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            WinitEventContext {
                app: &mut self.app,
                services,
                window,
                state: &mut state.user,
            },
            event,
        );
    }

    pub(super) fn deliver_platform_completion_now(
        &mut self,
        window: fret_core::AppWindowId,
        completion: PlatformCompletion,
    ) {
        match completion {
            PlatformCompletion::ClipboardText { token, text } => {
                self.deliver_window_event_now(window, &Event::ClipboardText { token, text });
            }
            PlatformCompletion::ClipboardTextUnavailable { token } => {
                self.deliver_window_event_now(
                    window,
                    &Event::ClipboardTextUnavailable {
                        token,
                        message: None,
                    },
                );
            }
            PlatformCompletion::ExternalDropData(data) => {
                self.deliver_window_event_now(window, &Event::ExternalDropData(data));
            }
            PlatformCompletion::FileDialogSelection(selection) => {
                self.deliver_window_event_now(window, &Event::FileDialogSelection(selection));
            }
            PlatformCompletion::FileDialogData(data) => {
                self.deliver_window_event_now(window, &Event::FileDialogData(data));
            }
            PlatformCompletion::FileDialogCanceled => {
                self.deliver_window_event_now(window, &Event::FileDialogCanceled);
            }
        }
    }

    fn dispatch_internal_drag_event(
        &mut self,
        window: fret_core::AppWindowId,
        pointer_id: fret_core::PointerId,
        kind: InternalDragKind,
        position: Point,
    ) {
        let Some(state) = self.windows.get_mut(window) else {
            return;
        };
        let modifiers = state.platform.input.modifiers;
        let services = Self::ui_services_mut(&mut self.renderer, &mut self.no_services);
        self.driver.handle_event(
            WinitEventContext {
                app: &mut self.app,
                services,
                window,
                state: &mut state.user,
            },
            &Event::InternalDrag(InternalDragEvent {
                pointer_id,
                position,
                kind,
                modifiers,
            }),
        );
    }

    pub(super) fn clear_internal_drag_hover_if_needed(&mut self) -> bool {
        let Some(window) = self.internal_drag_hover_window else {
            return false;
        };
        if self.dock_drag_pointer_id().is_some() {
            return false;
        }
        let pointer_id = self
            .internal_drag_pointer_id
            .take()
            .unwrap_or(fret_core::PointerId(0));
        self.internal_drag_hover_window = None;
        let pos = self.internal_drag_hover_pos.take().unwrap_or_default();
        self.dispatch_internal_drag_event(window, pointer_id, InternalDragKind::Cancel, pos);
        true
    }

    pub(super) fn route_internal_drag_hover_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        self.macos_refresh_cursor_screen_pos_for_dock_drag();

        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return self.clear_internal_drag_hover_if_needed();
        };
        let Some((drag_kind, drag_source_window, cross_window_hover)) = self
            .app
            .drag(pointer_id)
            .map(|d| (d.kind, d.source_window, d.cross_window_hover))
        else {
            return self.clear_internal_drag_hover_if_needed();
        };
        if !cross_window_hover {
            return self.clear_internal_drag_hover_if_needed();
        }

        let Some(screen_pos) = self.cursor_screen_pos else {
            return false;
        };

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let allow_window_under_cursor =
            caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None;

        // When a dock tear-off window is following the cursor, the cursor is always "inside" that
        // moving window. Prefer other windows under the cursor so we can dock back into the main
        // window (ImGui-style).
        let prefer_not = self.dock_tearoff_follow.map(|f| f.window);

        // Prefer the window we already hovered, if the cursor is still inside it. This makes
        // cross-window drag hover stable even when OS windows overlap and we don't have z-order.
        let hovered = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| {
                allow_window_under_cursor
                    .then(|| self.window_under_cursor(screen_pos, prefer_not))
                    .flatten()
            });
        let hovered = hovered.or_else(|| {
            // For dock tear-off, keep delivering `InternalDrag::Over` to the source window even
            // when the cursor is outside all windows so the UI can react before mouse-up.
            (drag_kind == fret_app::DRAG_KIND_DOCK_PANEL)
                .then_some(drag_source_window)
                .filter(|w| self.windows.contains_key(*w))
        });
        if hovered != self.internal_drag_hover_window {
            if let Some(prev) = self.internal_drag_hover_window.take() {
                let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
                self.dispatch_internal_drag_event(
                    prev,
                    pointer_id,
                    InternalDragKind::Leave,
                    prev_pos,
                );
            }

            if let Some(next) = hovered
                && let Some(pos) = self.local_pos_for_window(next, screen_pos)
            {
                self.dispatch_internal_drag_event(next, pointer_id, InternalDragKind::Enter, pos);
                self.internal_drag_hover_window = Some(next);
                self.internal_drag_hover_pos = Some(pos);
                self.internal_drag_pointer_id = Some(pointer_id);
            }
        }

        let Some(current) = self.internal_drag_hover_window else {
            return false;
        };
        let Some(pos) = self.local_pos_for_window(current, screen_pos) else {
            return false;
        };

        if drag_kind == fret_app::DRAG_KIND_DOCK_PANEL
            && std::env::var_os("FRET_DOCK_TEAROFF_LOG").is_some()
            && let Some(state) = self.windows.get(current)
        {
            let size_phys = state.window.surface_size();
            let scale = state.window.scale_factor();
            let size_logical: winit::dpi::LogicalSize<f32> = size_phys.to_logical(scale);
            let margin = 32.0f32;
            let oob = pos.x.0 < -margin
                || pos.y.0 < -margin
                || pos.x.0 > size_logical.width + margin
                || pos.y.0 > size_logical.height + margin;
            if oob {
                let outer = state.window.outer_position().ok();
                let deco = state.window.surface_position();
                dock_tearoff_log(format_args!(
                    "[cursor-oob] window={:?} screen=({:.1},{:.1}) local=({:.1},{:.1}) size=({:.1},{:.1}) scale={:.3} outer={:?} deco=({},{})",
                    current,
                    screen_pos.x,
                    screen_pos.y,
                    pos.x.0,
                    pos.y.0,
                    size_logical.width,
                    size_logical.height,
                    scale,
                    outer,
                    deco.x,
                    deco.y,
                ));
            }
        }

        if let Some(d) = self.app.drag_mut(pointer_id) {
            d.current_window = current;
            d.position = pos;
        }

        self.internal_drag_hover_pos = Some(pos);
        self.dispatch_internal_drag_event(current, pointer_id, InternalDragKind::Over, pos);
        true
    }

    pub(super) fn route_internal_drag_drop_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        self.macos_refresh_cursor_screen_pos_for_dock_drag();

        let Some(pointer_id) = self.dock_drag_pointer_id() else {
            return false;
        };
        let Some((drag_kind, drag_source_window, cross_window_hover)) = self
            .app
            .drag(pointer_id)
            .map(|d| (d.kind, d.source_window, d.cross_window_hover))
        else {
            return false;
        };
        if !cross_window_hover {
            return false;
        }

        let screen_pos = self
            .cursor_screen_pos
            .or_else(|| self.cursor_screen_pos_fallback_for_window(drag_source_window));
        let Some(screen_pos) = screen_pos else {
            return false;
        };

        let caps = self
            .app
            .global::<PlatformCapabilities>()
            .cloned()
            .unwrap_or_default();
        let allow_window_under_cursor =
            caps.ui.window_hover_detection != fret_runtime::WindowHoverDetectionQuality::None;

        let prefer_not = self.dock_tearoff_follow.map(|f| f.window);

        // Prefer the last hovered window if possible; window overlap makes hit-testing ambiguous.
        let target = self
            .internal_drag_hover_window
            .filter(|w| self.screen_pos_in_window(*w, screen_pos))
            .filter(|w| Some(*w) != prefer_not)
            .or_else(|| {
                allow_window_under_cursor
                    .then(|| self.window_under_cursor(screen_pos, prefer_not))
                    .flatten()
            })
            .or(self.internal_drag_hover_window);
        // If the cursor is outside all windows (Unity/ImGui-style tear-off), still deliver the
        // drop to the source window using the last known screen cursor position.
        let target = target.unwrap_or(drag_source_window);
        let pos = self.local_pos_for_window(target, screen_pos).or_else(|| {
            if self.internal_drag_hover_window == Some(target) {
                self.internal_drag_hover_pos
            } else {
                None
            }
        });
        let Some(pos) = pos else {
            return false;
        };

        if drag_kind == fret_app::DRAG_KIND_DOCK_PANEL
            && target != drag_source_window
            && let Some(runtime) = self.windows.get(target)
        {
            let sender = self
                .windows
                .get(drag_source_window)
                .map(|w| w.window.as_ref());
            let _ = bring_window_to_front(runtime.window.as_ref(), sender);
        }

        if let Some(prev) = self.internal_drag_hover_window.take()
            && prev != target
        {
            let prev_pos = self.internal_drag_hover_pos.take().unwrap_or_default();
            self.dispatch_internal_drag_event(prev, pointer_id, InternalDragKind::Leave, prev_pos);
        }
        self.internal_drag_hover_window = Some(target);
        self.internal_drag_hover_pos = Some(pos);
        self.internal_drag_pointer_id = Some(pointer_id);

        if let Some(d) = self.app.drag_mut(pointer_id) {
            d.current_window = target;
            d.position = pos;
        }

        self.dispatch_internal_drag_event(target, pointer_id, InternalDragKind::Drop, pos);
        true
    }
}
