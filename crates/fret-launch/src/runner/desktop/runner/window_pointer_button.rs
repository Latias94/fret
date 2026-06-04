use fret_core::time::Instant;
use fret_core::{AppWindowId, Event};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_pointer_button(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        ev: &WindowEvent,
    ) {
        let (mapped, _scale_factor) = {
            let Some(runtime) = self.windows.get_mut(app_window) else {
                return;
            };
            let mut mapped = Vec::new();
            runtime
                .platform
                .handle_window_event(runtime.window.scale_factor(), ev, &mut mapped);
            (mapped, runtime.window.scale_factor())
        };

        if let Some(p) = self.cursor_screen_pos_fallback_for_window(app_window) {
            self.cursor_screen_pos = Some(p);
            #[cfg(target_os = "macos")]
            self.macos_calibrate_cursor_transform_from_window_sample(p, _scale_factor);
        }

        self.sync_dock_drag_pointer_capture();

        let dock_drag_capture = self
            .dock_drag_pointer_id()
            .and_then(|pointer_id| {
                self.app
                    .drag(pointer_id)
                    .map(|d| (pointer_id, d.source_window))
            })
            .filter(|(_pointer_id, window)| self.windows.contains_key(*window));
        let dock_drag_capture_pos = self.cursor_screen_pos;

        let mut saw_left_down = false;
        let mut saw_left_up = false;
        let mut left_up_pointer_id: Option<fret_core::PointerId> = None;
        for evt in &mapped {
            let Event::Pointer(pointer) = evt else {
                continue;
            };
            match pointer {
                fret_core::PointerEvent::Down {
                    button: fret_core::MouseButton::Left,
                    pointer_id: _,
                    ..
                } => {
                    saw_left_down = true;
                }
                fret_core::PointerEvent::Up {
                    button: fret_core::MouseButton::Left,
                    pointer_id,
                    ..
                } => {
                    saw_left_up = true;
                    left_up_pointer_id = Some(*pointer_id);
                }
                _ => {}
            }
        }

        if saw_left_down {
            self.left_mouse_down = true;
        }

        if saw_left_up {
            self.left_mouse_down = false;
            self.saw_left_mouse_release_this_turn = true;
            let cancel_pointer_id = self.internal_drag_routing_pointer_id();
            // Deliver the cursor-based drop on any left mouse release; this keeps docking
            // robust even when the drag pointer id is not `PointerId(0)`.
            self.route_internal_drag_drop_from_cursor();
            if self.dock_tearoff_follow.is_some() {
                self.stop_dock_tearoff_follow(Instant::now(), true);
            }

            // Cross-window drags are runner-routed (Enter/Over/Drop), so ensure the
            // drag session cannot get "stuck" if no widget ends it.
            if let Some(released) = cancel_pointer_id.or(left_up_pointer_id)
                && self
                    .app
                    .drag(released)
                    .is_some_and(|d| d.cross_window_hover)
            {
                self.app.cancel_drag(released);
                let _ = self.clear_internal_drag_hover_if_needed();
            }
        }

        for evt in mapped {
            let mut delivered = false;
            if let Some((dock_pointer_id, dock_source_window)) = dock_drag_capture
                && dock_source_window != app_window
            {
                match &evt {
                    Event::Pointer(fret_core::PointerEvent::Up {
                        pointer_id,
                        position: _,
                        button,
                        modifiers,
                        is_click,
                        click_count,
                        pointer_type,
                    }) if *pointer_id == dock_pointer_id => {
                        let pos = dock_drag_capture_pos
                            .and_then(|screen| {
                                self.local_pos_for_window(dock_source_window, screen)
                            })
                            .or_else(|| {
                                self.windows
                                    .get(dock_source_window)
                                    .map(|w| w.platform.input.cursor_pos)
                            })
                            .unwrap_or_default();
                        self.deliver_window_event_now(
                            dock_source_window,
                            &Event::Pointer(fret_core::PointerEvent::Up {
                                pointer_id: *pointer_id,
                                position: pos,
                                button: *button,
                                modifiers: *modifiers,
                                is_click: *is_click,
                                click_count: *click_count,
                                pointer_type: *pointer_type,
                            }),
                        );
                        delivered = true;
                    }
                    Event::Pointer(fret_core::PointerEvent::Down {
                        pointer_id,
                        position: _,
                        button,
                        modifiers,
                        click_count,
                        pointer_type,
                    }) if *pointer_id == dock_pointer_id => {
                        let pos = dock_drag_capture_pos
                            .and_then(|screen| {
                                self.local_pos_for_window(dock_source_window, screen)
                            })
                            .or_else(|| {
                                self.windows
                                    .get(dock_source_window)
                                    .map(|w| w.platform.input.cursor_pos)
                            })
                            .unwrap_or_default();
                        self.deliver_window_event_now(
                            dock_source_window,
                            &Event::Pointer(fret_core::PointerEvent::Down {
                                pointer_id: *pointer_id,
                                position: pos,
                                button: *button,
                                modifiers: *modifiers,
                                click_count: *click_count,
                                pointer_type: *pointer_type,
                            }),
                        );
                        delivered = true;
                    }
                    _ => {}
                }
            }
            if !delivered {
                self.deliver_window_event_now(app_window, &evt);
            }
        }
        self.drain_effects(event_loop);
    }
}
