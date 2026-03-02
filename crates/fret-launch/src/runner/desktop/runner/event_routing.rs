use super::*;
use std::fmt;

fn diag_dock_drag_trace(args: fmt::Arguments<'_>) {
    use std::{
        io::Write as _,
        sync::{Mutex, OnceLock},
    };

    if std::env::var_os("FRET_DOCK_DRAG_TRACE").is_none() {
        return;
    }

    static LOG_FILE: OnceLock<Mutex<std::fs::File>> = OnceLock::new();
    let file = LOG_FILE.get_or_init(|| {
        let out_dir = std::env::var_os("FRET_DIAG_DIR")
            .filter(|v| !v.is_empty())
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("target").join("fret-diag"));
        let _ = std::fs::create_dir_all(&out_dir);
        let path = out_dir.join("dock_drag_runtime_trace.log");
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open dock_drag_runtime_trace.log");
        Mutex::new(file)
    });
    let Ok(mut file) = file.lock() else {
        return;
    };
    let _ = writeln!(file, "{}", args);
}

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn internal_drag_routing_pointer_id(&self) -> Option<fret_core::PointerId> {
        if let Some(pointer_id) = self.dock_drag_pointer_id() {
            return Some(pointer_id);
        }
        use fret_runtime::DragHost as _;
        self.app.find_drag_pointer_id(|d| d.cross_window_hover)
    }

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
    ) -> bool {
        let Some(state) = self.windows.get_mut(window) else {
            return false;
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
        true
    }

    pub(super) fn clear_internal_drag_hover_if_needed(&mut self) -> bool {
        let Some(window) = self.internal_drag_hover_window else {
            return false;
        };
        if self.internal_drag_routing_pointer_id().is_some() {
            return false;
        }
        let pointer_id = self
            .internal_drag_pointer_id
            .take()
            .unwrap_or(fret_core::PointerId(0));
        self.internal_drag_hover_window = None;
        let pos = self.internal_drag_hover_pos.take().unwrap_or_default();
        let _ =
            self.dispatch_internal_drag_event(window, pointer_id, InternalDragKind::Cancel, pos);
        true
    }

    pub(super) fn route_internal_drag_hover_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        {
            if !self.diag_pointer_input_isolation_active() {
                self.macos_refresh_cursor_screen_pos_for_dock_drag();
            }
        }

        let Some(pointer_id) = self.internal_drag_routing_pointer_id() else {
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
        let reliable_window_under_cursor =
            caps.ui.window_hover_detection == fret_runtime::WindowHoverDetectionQuality::Reliable;

        let mut moving_window = self
            .dock_tearoff_follow
            .filter(|follow| follow.source_window == drag_source_window)
            .map(|follow| follow.window)
            .or_else(|| {
                matches!(
                    drag_kind,
                    fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                )
                .then_some(drag_source_window)
                .filter(|w| self.main_window.is_some_and(|main| *w != main))
            });
        let peek_behind_moving_window = self.dock_tearoff_follow.is_some_and(|follow| {
            follow.source_window == drag_source_window && follow.transparent_payload_applied
        });
        let prefer_not = peek_behind_moving_window.then_some(moving_window).flatten();

        let mut window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Unknown;
        let mut window_under_moving_window = None;
        let mut window_under_moving_window_source = fret_runtime::WindowUnderCursorSource::Unknown;
        if let Some(moving_window) = moving_window {
            let diag_cursor_override_active = self.diag_pointer_input_isolation_active();
            if allow_window_under_cursor {
                // When scripted diagnostics inject cursor overrides in window-client coordinates,
                // the simulated cursor may temporarily drift outside the moving window while the
                // runner is also updating OS window positions (tear-off follow).
                //
                // Prefer sampling a few stable points inside the moving window to recover the
                // "window under moving window" result that an OS cursor would report.
                let mut candidates = Vec::with_capacity(3);
                candidates.push(screen_pos);
                if diag_cursor_override_active {
                    if let Some(clamped) =
                        self.clamp_screen_pos_to_window_client(moving_window, screen_pos)
                    {
                        candidates.push(clamped);
                    }
                    if let Some((origin, size)) = self.window_client_rect_screen(moving_window) {
                        candidates.push(winit::dpi::PhysicalPosition::new(
                            origin.x + (size.width as f64) * 0.5,
                            origin.y + (size.height as f64) * 0.5,
                        ));
                    }
                }

                for candidate in candidates {
                    let hit = if reliable_window_under_cursor {
                        self.window_under_cursor_platform(candidate, Some(moving_window))
                    } else {
                        self.window_under_cursor_best_effort(candidate, Some(moving_window))
                    };
                    if matches!(
                        window_under_moving_window_source,
                        fret_runtime::WindowUnderCursorSource::Unknown
                    ) && !matches!(hit.source, fret_runtime::WindowUnderCursorSource::Unknown)
                    {
                        window_under_moving_window_source = hit.source;
                    }
                    if let Some(w) = hit.window.filter(|w| *w != moving_window) {
                        window_under_moving_window_source = hit.source;
                        window_under_moving_window = Some(w);
                        break;
                    }
                }
            }
        }
        let hovered = if reliable_window_under_cursor {
            if allow_window_under_cursor {
                let hit = self.window_under_cursor_platform(screen_pos, prefer_not);
                window_under_cursor_source = hit.source;
                hit.window
            } else {
                None
            }
        } else if peek_behind_moving_window {
            // When the runner applies ImGui-style transparent payload (best-effort), prefer
            // selecting the "window under moving window" for hover routing. This enables
            // docking-back interactions when the moving window overlaps a potential target.
            if allow_window_under_cursor {
                let hit = self.window_under_cursor_best_effort(screen_pos, prefer_not);
                window_under_cursor_source = hit.source;
                hit.window
            } else {
                None
            }
        } else {
            // Prefer the window we already hovered, if the cursor is still inside it. This makes
            // cross-window drag hover stable even when OS windows overlap and we don't have
            // reliable z-order.
            self.internal_drag_hover_window
                .filter(|w| self.screen_pos_in_window(*w, screen_pos))
                .inspect(|_| {
                    window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Latched;
                })
                .or_else(|| {
                    if !allow_window_under_cursor {
                        return None;
                    }
                    let hit = self.window_under_cursor_best_effort(screen_pos, None);
                    window_under_cursor_source = hit.source;
                    hit.window
                })
        };

        // ImGui-style "peek behind moving window" hover routing:
        //
        // When a dock payload window overlaps a target, docking previews/resolve should be able
        // to treat the window under the moving window as the hover target (ImGui's
        // `HoveredWindowUnderMovingWindow`), rather than getting "stuck" on the moving window.
        //
        // In Fret this is primarily driven by transparent payload / follow-window behavior.
        let follow_active = self.dock_tearoff_follow.is_some_and(|follow| {
            follow.source_window == drag_source_window && follow.manual_follow
        });
        let wants_transparent_payload = {
            let settings = self
                .app
                .global::<fret_runtime::DockingInteractionSettings>()
                .copied()
                .unwrap_or_default();
            settings.transparent_payload_during_follow
                || std::env::var_os("FRET_DOCK_TEAROFF_TRANSPARENT_PAYLOAD").is_some()
        };
        let hover_under_moving_window = follow_active
            || (wants_transparent_payload
                && matches!(
                    drag_kind,
                    fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                ))
            || std::env::var_os("FRET_DOCK_DRAG_HOVER_UNDER_MOVING_WINDOW").is_some();
        let hovered = if hover_under_moving_window
            && allow_window_under_cursor
            && window_under_moving_window.is_some()
        {
            window_under_cursor_source = window_under_moving_window_source;
            window_under_moving_window
        } else {
            hovered
        };

        // Note: by default we keep `hovered` as the OS-selected window under the cursor.
        // `window_under_moving_window` is always reported separately so diagnostics (and future
        // policy) can distinguish "HoveredWindow" vs "HoveredWindowUnderMovingWindow" (ImGui
        // terminology). The env gate above allows opt-in routing to the under-moving window.

        // When the cursor is outside all windows, prefer latching to whichever window we were
        // already hovering. This makes scripted cross-window drags deterministic even if the
        // last injected cursor position overshoots the intended target window.
        let hovered = hovered
            .or_else(|| {
                self.internal_drag_hover_window
                    .filter(|w| self.windows.contains_key(*w))
            })
            .or_else(|| {
                // Fallback: keep delivering `InternalDrag::Over` to the source window so the UI
                // can react before mouse-up (tear-off / out-of-bounds heuristics).
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
                let _ = self.dispatch_internal_drag_event(
                    next,
                    pointer_id,
                    InternalDragKind::Enter,
                    pos,
                );
                self.internal_drag_hover_window = Some(next);
                self.internal_drag_hover_pos = Some(pos);
                self.internal_drag_pointer_id = Some(pointer_id);
            }
        }

        let Some(current) = self.internal_drag_hover_window else {
            return false;
        };
        let diag_override_active = self.diag_pointer_input_isolation_active();

        let screen_pos_for_pos = if diag_override_active && current != drag_source_window {
            self.clamp_screen_pos_to_window_client(current, screen_pos)
                .unwrap_or(screen_pos)
        } else {
            screen_pos
        };
        let Some(pos) = self.local_pos_for_window(current, screen_pos_for_pos) else {
            return false;
        };

        // Best-effort diagnostics: ensure `moving_window` is set for dock drags inside a non-main
        // window even if the drag source window differs from the hovered window (can happen under
        // overlap + scripted injection).
        if moving_window.is_none()
            && matches!(
                drag_kind,
                fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
            )
            && self.main_window.is_some_and(|main| current != main)
        {
            moving_window = Some(current);
            window_under_moving_window = None;
            window_under_moving_window_source = fret_runtime::WindowUnderCursorSource::Unknown;
            if allow_window_under_cursor {
                let diag_cursor_override_active = self.diag_pointer_input_isolation_active();
                let mut candidates = Vec::with_capacity(3);
                candidates.push(screen_pos);
                if diag_cursor_override_active {
                    if let Some(clamped) =
                        self.clamp_screen_pos_to_window_client(current, screen_pos)
                    {
                        candidates.push(clamped);
                    }
                    if let Some((origin, size)) = self.window_client_rect_screen(current) {
                        candidates.push(winit::dpi::PhysicalPosition::new(
                            origin.x + (size.width as f64) * 0.5,
                            origin.y + (size.height as f64) * 0.5,
                        ));
                    }
                }

                for candidate in candidates {
                    let hit = if reliable_window_under_cursor {
                        self.window_under_cursor_platform(candidate, Some(current))
                    } else {
                        self.window_under_cursor_best_effort(candidate, Some(current))
                    };
                    if matches!(
                        window_under_moving_window_source,
                        fret_runtime::WindowUnderCursorSource::Unknown
                    ) && !matches!(hit.source, fret_runtime::WindowUnderCursorSource::Unknown)
                    {
                        window_under_moving_window_source = hit.source;
                    }
                    if let Some(w) = hit.window.filter(|w| *w != current) {
                        window_under_moving_window_source = hit.source;
                        window_under_moving_window = Some(w);
                        break;
                    }
                }
            }
        }

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
            d.window_under_cursor_source = window_under_cursor_source;
            d.moving_window = moving_window;
            d.window_under_moving_window = window_under_moving_window;
            d.window_under_moving_window_source = window_under_moving_window_source;
        }

        #[cfg(target_os = "windows")]
        let moving_rect = moving_window
            .and_then(|w| self.windows.get(w))
            .and_then(|state| Self::hwnd_for_window(state.window.as_ref()))
            .and_then(super::win32::window_rect_screen_for_hwnd);
        #[cfg(target_os = "windows")]
        let main_rect = self
            .main_window
            .and_then(|w| self.windows.get(w))
            .and_then(|state| Self::hwnd_for_window(state.window.as_ref()))
            .and_then(super::win32::window_rect_screen_for_hwnd);
        #[cfg(not(target_os = "windows"))]
        let moving_rect: Option<(i32, i32, i32, i32)> = None;
        #[cfg(not(target_os = "windows"))]
        let main_rect: Option<(i32, i32, i32, i32)> = None;

        diag_dock_drag_trace(format_args!(
            "[hover] tick={} pointer={:?} kind={:?} src={:?} cur={:?} moving={:?} under_moving={:?} cursor_src={:?} under_src={:?} screen=({:.1},{:.1}) local=({:.1},{:.1}) moving_rect={:?} main_rect={:?}",
            self.tick_id.0,
            pointer_id,
            drag_kind,
            drag_source_window,
            current,
            moving_window,
            window_under_moving_window,
            window_under_cursor_source,
            window_under_moving_window_source,
            screen_pos.x,
            screen_pos.y,
            pos.x.0,
            pos.y.0,
            moving_rect,
            main_rect,
        ));

        self.internal_drag_hover_pos = Some(pos);
        let _ = self.dispatch_internal_drag_event(current, pointer_id, InternalDragKind::Over, pos);
        if let Some(state) = self.windows.get(current) {
            state.window.request_redraw();
        }
        // Keep both the drag source and the hovered/target window rendering while a cross-window
        // dock drag is active. This prevents diagnostics scripts that are still attached to the
        // payload window from stalling on `wait_frames` when hover routing peeks behind overlap.
        for w in [drag_source_window, moving_window.unwrap_or(drag_source_window)] {
            if w == current {
                continue;
            }
            if let Some(state) = self.windows.get(w) {
                state.window.request_redraw();
            }
        }
        true
    }

    pub(super) fn route_internal_drag_drop_from_cursor(&mut self) -> bool {
        #[cfg(target_os = "macos")]
        {
            if !self.diag_pointer_input_isolation_active() {
                self.macos_refresh_cursor_screen_pos_for_dock_drag();
            }
        }

        let Some(pointer_id) = self.internal_drag_routing_pointer_id() else {
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
        let reliable_window_under_cursor =
            caps.ui.window_hover_detection == fret_runtime::WindowHoverDetectionQuality::Reliable;

        let mut moving_window = self
            .dock_tearoff_follow
            .filter(|follow| follow.source_window == drag_source_window)
            .map(|follow| follow.window)
            .or_else(|| {
                matches!(
                    drag_kind,
                    fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
                )
                .then_some(drag_source_window)
                .filter(|w| self.main_window.is_some_and(|main| *w != main))
            });
        let peek_behind_moving_window = self.dock_tearoff_follow.is_some_and(|follow| {
            follow.source_window == drag_source_window && follow.transparent_payload_applied
        });
        let prefer_not = peek_behind_moving_window.then_some(moving_window).flatten();

        let mut window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Unknown;
        let mut window_under_moving_window = None;
        let mut window_under_moving_window_source = fret_runtime::WindowUnderCursorSource::Unknown;
        if let Some(moving_window) = moving_window {
            let diag_cursor_override_active = self.diag_pointer_input_isolation_active();
            if allow_window_under_cursor {
                let mut candidates = Vec::with_capacity(3);
                candidates.push(screen_pos);
                if diag_cursor_override_active {
                    if let Some(clamped) =
                        self.clamp_screen_pos_to_window_client(moving_window, screen_pos)
                    {
                        candidates.push(clamped);
                    }
                    if let Some((origin, size)) = self.window_client_rect_screen(moving_window) {
                        candidates.push(winit::dpi::PhysicalPosition::new(
                            origin.x + (size.width as f64) * 0.5,
                            origin.y + (size.height as f64) * 0.5,
                        ));
                    }
                }

                for candidate in candidates {
                    let hit = if reliable_window_under_cursor {
                        self.window_under_cursor_platform(candidate, Some(moving_window))
                    } else {
                        self.window_under_cursor_best_effort(candidate, Some(moving_window))
                    };
                    if matches!(
                        window_under_moving_window_source,
                        fret_runtime::WindowUnderCursorSource::Unknown
                    ) && !matches!(hit.source, fret_runtime::WindowUnderCursorSource::Unknown)
                    {
                        window_under_moving_window_source = hit.source;
                    }
                    if let Some(w) = hit.window.filter(|w| *w != moving_window) {
                        window_under_moving_window_source = hit.source;
                        window_under_moving_window = Some(w);
                        break;
                    }
                }
            }
        }
        let mut target = if reliable_window_under_cursor {
            let mut out = None;
            if allow_window_under_cursor {
                let hit = self.window_under_cursor_platform(screen_pos, prefer_not);
                window_under_cursor_source = hit.source;
                out = hit.window;
            }
            if out.is_none()
                && let Some(w) = self.internal_drag_hover_window
            {
                out = Some(w);
                if window_under_cursor_source == fret_runtime::WindowUnderCursorSource::Unknown {
                    window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Latched;
                }
            }
            out
        } else if peek_behind_moving_window {
            let mut out = None;
            if allow_window_under_cursor {
                let hit = self.window_under_cursor_best_effort(screen_pos, prefer_not);
                window_under_cursor_source = hit.source;
                out = hit.window;
            }
            if out.is_none()
                && let Some(w) = self.internal_drag_hover_window
            {
                out = Some(w);
                if window_under_cursor_source == fret_runtime::WindowUnderCursorSource::Unknown {
                    window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Latched;
                }
            }
            out
        } else {
            // Prefer the last hovered window if possible; window overlap makes hit-testing
            // ambiguous when we don't have reliable z-order.
            let mut out = self
                .internal_drag_hover_window
                .filter(|w| self.screen_pos_in_window(*w, screen_pos));
            if out.is_some() {
                window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Latched;
            } else if allow_window_under_cursor {
                let hit = self.window_under_cursor_best_effort(screen_pos, None);
                window_under_cursor_source = hit.source;
                out = hit.window;
            }
            out.or_else(|| {
                self.internal_drag_hover_window.inspect(|_| {
                    if window_under_cursor_source == fret_runtime::WindowUnderCursorSource::Unknown
                    {
                        window_under_cursor_source = fret_runtime::WindowUnderCursorSource::Latched;
                    }
                })
            })
        };
        // Keep drop routing consistent with hover routing: when a dock-floating OS window is
        // following the cursor, prefer the window under the moving window so overlap cases can
        // dock back without requiring transparent payload / mouse passthrough.
        if matches!(
            drag_kind,
            fret_app::DRAG_KIND_DOCK_PANEL | fret_runtime::DRAG_KIND_DOCK_TABS
        ) && let Some(moving_window) = moving_window
            && window_under_moving_window.is_some()
        {
            // Only treat the "window under moving window" as the drop target when the cursor is
            // actually inside (or very near) the moving window. During tear-off, scripts (and
            // sometimes OS scheduling) can temporarily report cursor positions outside all windows
            // while the moving window is still being positioned; dropping in that state should
            // finalize the tear-off (drop to the moving/source window), not immediately re-dock
            // into the overlapped window.
            let cursor_in_moving_window = self.screen_pos_in_window(moving_window, screen_pos);
            let cursor_near_moving_window = self
                .window_client_rect_screen(moving_window)
                .is_some_and(|(origin, size)| {
                    // Small tolerance for 1-2 frame drift during scripted cursor overrides.
                    let margin = 4.0;
                    let max_x = origin.x + (size.width as f64) + margin;
                    let max_y = origin.y + (size.height as f64) + margin;
                    let min_x = origin.x - margin;
                    let min_y = origin.y - margin;
                    screen_pos.x >= min_x
                        && screen_pos.x <= max_x
                        && screen_pos.y >= min_y
                        && screen_pos.y <= max_y
                });
            if cursor_in_moving_window || cursor_near_moving_window {
                target = window_under_moving_window;
                window_under_cursor_source = window_under_moving_window_source;
            }
        }

        // If the cursor is outside all windows (Unity/ImGui-style tear-off), still deliver the
        // drop to the source window using the last known screen cursor position.
        let target = target.unwrap_or(drag_source_window);
        let diag_override_active = self.diag_pointer_input_isolation_active();

        let screen_pos_for_pos = if diag_override_active && target != drag_source_window {
            self.clamp_screen_pos_to_window_client(target, screen_pos)
                .unwrap_or(screen_pos)
        } else {
            screen_pos
        };

        let pos = self
            .local_pos_for_window(target, screen_pos_for_pos)
            .or_else(|| {
                if self.internal_drag_hover_window == Some(target) {
                    self.internal_drag_hover_pos
                } else {
                    None
                }
            });
        let Some(pos) = pos else {
            return false;
        };

        if moving_window.is_none()
            && matches!(
                drag_kind,
                fret_runtime::DRAG_KIND_DOCK_TABS | fret_runtime::DRAG_KIND_DOCK_PANEL
            )
            && self.main_window.is_some_and(|main| target != main)
        {
            moving_window = Some(target);
            window_under_moving_window = None;
            window_under_moving_window_source = fret_runtime::WindowUnderCursorSource::Unknown;
            if allow_window_under_cursor {
                let diag_cursor_override_active = self.diag_pointer_input_isolation_active();
                let mut candidates = Vec::with_capacity(3);
                candidates.push(screen_pos);
                if diag_cursor_override_active {
                    if let Some(clamped) =
                        self.clamp_screen_pos_to_window_client(target, screen_pos)
                    {
                        candidates.push(clamped);
                    }
                    if let Some((origin, size)) = self.window_client_rect_screen(target) {
                        candidates.push(winit::dpi::PhysicalPosition::new(
                            origin.x + (size.width as f64) * 0.5,
                            origin.y + (size.height as f64) * 0.5,
                        ));
                    }
                }

                for candidate in candidates {
                    let hit = if reliable_window_under_cursor {
                        self.window_under_cursor_platform(candidate, Some(target))
                    } else {
                        self.window_under_cursor_best_effort(candidate, Some(target))
                    };
                    if matches!(
                        window_under_moving_window_source,
                        fret_runtime::WindowUnderCursorSource::Unknown
                    ) && !matches!(hit.source, fret_runtime::WindowUnderCursorSource::Unknown)
                    {
                        window_under_moving_window_source = hit.source;
                    }
                    if let Some(w) = hit.window.filter(|w| *w != target) {
                        window_under_moving_window_source = hit.source;
                        window_under_moving_window = Some(w);
                        break;
                    }
                }
            }
        }

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
            d.window_under_cursor_source = window_under_cursor_source;
            d.moving_window = moving_window;
            d.window_under_moving_window = window_under_moving_window;
            d.window_under_moving_window_source = window_under_moving_window_source;
        }
        diag_dock_drag_trace(format_args!(
            "[drop] tick={} pointer={:?} kind={:?} src={:?} target={:?} local=({:.1},{:.1}) moving={:?} under_moving={:?} cursor_src={:?} under_src={:?}",
            self.tick_id.0,
            pointer_id,
            drag_kind,
            drag_source_window,
            target,
            pos.x.0,
            pos.y.0,
            moving_window,
            window_under_moving_window,
            window_under_cursor_source,
            window_under_moving_window_source,
        ));

        let dispatched =
            self.dispatch_internal_drag_event(target, pointer_id, InternalDragKind::Drop, pos);
        diag_dock_drag_trace(format_args!(
            "[drop-dispatch] tick={} pointer={:?} target={:?} ok={}",
            self.tick_id.0, pointer_id, target, dispatched
        ));
        if let Some(state) = self.windows.get(target) {
            state.window.request_redraw();
        }
        if target != drag_source_window
            && let Some(state) = self.windows.get(drag_source_window)
        {
            state.window.request_redraw();
        }

        // Cross-window dock drags are runner-routed (Enter/Over/Drop). Ensure the drag session
        // cannot get stuck if the pointer release is delivered to a different window than the
        // original drag source (common in diagnostics injection and multi-window tear-off flows).
        if self
            .app
            .drag(pointer_id)
            .is_some_and(|d| d.cross_window_hover)
        {
            self.app.cancel_drag(pointer_id);
            let _ = self.clear_internal_drag_hover_if_needed();
        }
        if self.dock_tearoff_follow.is_some() {
            self.stop_dock_tearoff_follow(std::time::Instant::now(), true);
        }
        true
    }
}
