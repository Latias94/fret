use fret_core::{AppWindowId, Event, ExternalDragEvent, ExternalDragKind};
use winit::{
    dpi::PhysicalPosition,
    event::{PointerSource, WindowEvent},
    event_loop::ActiveEventLoop,
};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_pointer_moved(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        ev: &WindowEvent,
    ) {
        let (mapped, pos, external_drag_token, screen_pos, _scale_factor) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                return;
            };

            let mut mapped = Vec::new();
            state
                .platform
                .handle_window_event(state.window.scale_factor(), ev, &mut mapped);

            let pos = state.platform.input.cursor_pos;
            let external_drag_token = state.external_drag_token;
            let scale_factor = state.window.scale_factor();
            let screen_pos = match ev {
                WindowEvent::PointerMoved {
                    position, source, ..
                } if !matches!(source, PointerSource::Touch { .. }) => {
                    state.window.outer_position().ok().map(|outer| {
                        let surface = state.window.surface_position();
                        PhysicalPosition::new(
                            outer.x as f64 + surface.x as f64 + position.x,
                            outer.y as f64 + surface.y as f64 + position.y,
                        )
                    })
                }
                _ => None,
            };

            (mapped, pos, external_drag_token, screen_pos, scale_factor)
        };

        let suppress_os_cursor_sample = self.diag_pointer_input_isolation_active();

        if !suppress_os_cursor_sample {
            if let Some(p) = screen_pos {
                self.cursor_screen_pos = Some(p);
                #[cfg(target_os = "macos")]
                self.macos_calibrate_cursor_transform_from_window_sample(p, _scale_factor);
            }

            let _ = self.update_dock_tearoff_follow();
        }

        let dock_drag_capture = self
            .dock_drag_pointer_id()
            .and_then(|pointer_id| {
                self.app
                    .drag(pointer_id)
                    .map(|d| (pointer_id, d.source_window))
            })
            .filter(|(_pointer_id, window)| self.windows.contains_key(*window));

        if let Some(token) = external_drag_token {
            let paths = self.external_drop.paths(token).unwrap_or(&[]);
            let kind =
                ExternalDragKind::OverFiles(fret_runner_winit::external_drag_files(token, paths));
            let evt = Event::ExternalDrag(ExternalDragEvent {
                position: pos,
                kind,
            });
            self.deliver_window_event_now(app_window, &evt);
        }

        for evt in mapped {
            let mut delivered = false;
            if let Some((dock_pointer_id, dock_source_window)) = dock_drag_capture
                && dock_source_window != app_window
                && let Event::Pointer(fret_core::PointerEvent::Move {
                    pointer_id,
                    position: _,
                    buttons,
                    modifiers,
                    pointer_type,
                }) = &evt
                && *pointer_id == dock_pointer_id
            {
                let pos = self
                    .cursor_screen_pos
                    .and_then(|screen| self.local_pos_for_window(dock_source_window, screen))
                    .or_else(|| {
                        self.windows
                            .get(dock_source_window)
                            .map(|w| w.platform.input.cursor_pos)
                    })
                    .unwrap_or(pos);
                self.deliver_window_event_now(
                    dock_source_window,
                    &Event::Pointer(fret_core::PointerEvent::Move {
                        pointer_id: *pointer_id,
                        position: pos,
                        buttons: *buttons,
                        modifiers: *modifiers,
                        pointer_type: *pointer_type,
                    }),
                );
                delivered = true;
            }
            if !delivered {
                self.deliver_window_event_now(app_window, &evt);
            }
        }

        self.sync_dock_drag_pointer_capture();
        self.route_internal_drag_hover_from_cursor();
        self.drain_effects(event_loop);
    }
}
