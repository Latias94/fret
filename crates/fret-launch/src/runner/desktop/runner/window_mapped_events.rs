use fret_core::time::Instant;
use fret_core::{AppWindowId, Event};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

use super::wheel_coalescing::wheel_coalesce_delta;
use super::window::PendingWheelEvent;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_mapped_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        ev: &WindowEvent,
    ) {
        let mapped = {
            let Some(state) = self.windows.get_mut(app_window) else {
                return;
            };
            let mut mapped = Vec::new();
            state
                .platform
                .handle_window_event(state.window.scale_factor(), ev, &mut mapped);
            mapped
        };

        let wheel_coalescing_enabled = Self::wheel_coalescing_enabled();
        let mut saw_wheel = false;
        let mapped = if wheel_coalescing_enabled {
            let mut passthrough = Vec::with_capacity(mapped.len());
            let mut pending: Option<PendingWheelEvent> = None;

            for evt in mapped {
                match evt {
                    Event::Pointer(fret_core::PointerEvent::Wheel {
                        pointer_id,
                        position,
                        delta,
                        modifiers,
                        pointer_type,
                    }) => {
                        saw_wheel = true;
                        pending = Some(match pending {
                            Some(mut prev) => {
                                prev.delta = wheel_coalesce_delta(prev.delta, delta);
                                prev.position = position;
                                prev.modifiers = modifiers;
                                prev.pointer_type = pointer_type;
                                prev.pointer_id = pointer_id;
                                prev
                            }
                            None => PendingWheelEvent {
                                pointer_id,
                                position,
                                delta,
                                modifiers,
                                pointer_type,
                            },
                        });
                    }
                    other => passthrough.push(other),
                }
            }

            if let Some(pending) = pending
                && let Some(state) = self.windows.get_mut(app_window)
            {
                state.pending_wheel = Some(match state.pending_wheel.take() {
                    Some(mut prev) => {
                        prev.delta = wheel_coalesce_delta(prev.delta, pending.delta);
                        prev.position = pending.position;
                        prev.modifiers = pending.modifiers;
                        prev.pointer_type = pending.pointer_type;
                        prev.pointer_id = pending.pointer_id;
                        prev
                    }
                    None => pending,
                });
            }

            passthrough
        } else {
            mapped
        };

        if saw_wheel {
            self.app.request_redraw(app_window);
        }

        if mapped.iter().any(|evt| {
            matches!(
                evt,
                Event::KeyDown {
                    key: fret_core::KeyCode::F12,
                    ..
                }
            )
        }) {
            if let Some(r) = self.renderdoc.as_mut() {
                r.request_capture();
                self.app.request_redraw(app_window);
            } else if std::env::var_os("FRET_RENDERDOC")
                .filter(|v| !v.is_empty())
                .is_some()
                || std::env::var_os("FRET_RENDERDOC_DLL")
                    .filter(|v| !v.is_empty())
                    .is_some()
            {
                tracing::warn!(
                    "renderdoc capture requested but renderdoc was not initialized (restart with renderdoc.dll available)"
                );
            }
        }

        // ADR 0072 (proposed): Escape cancels an active cross-window dock drag session.
        if mapped.iter().any(|evt| {
            matches!(
                evt,
                Event::KeyDown {
                    key: fret_core::KeyCode::Escape,
                    ..
                }
            )
        }) && self.dock_drag_pointer_id().is_some()
        {
            if let Some(pointer_id) = self.dock_drag_pointer_id() {
                self.app.cancel_drag(pointer_id);
            }
            let _ = self.clear_internal_drag_hover_if_needed();
            if self.dock_tearoff_follow.is_some() {
                self.stop_dock_tearoff_follow(Instant::now(), true);
            }
            self.drain_effects(event_loop);
            return;
        }

        for evt in mapped {
            self.deliver_window_event_now(app_window, &evt);
        }
        self.drain_effects(event_loop);
    }
}
