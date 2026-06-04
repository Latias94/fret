use std::path::PathBuf;

use fret_core::{AppWindowId, Event, ExternalDragEvent, ExternalDragKind};
use fret_platform::external_drop::ExternalDropProvider as _;
use winit::{dpi::PhysicalPosition, event_loop::ActiveEventLoop};

use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_drag_entered(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        paths: Vec<PathBuf>,
        position: PhysicalPosition<f64>,
    ) {
        tracing::debug!(count = paths.len(), "winit drag entered");
        let existing = self
            .windows
            .get(app_window)
            .and_then(|s| s.external_drag_token);
        let token = existing.unwrap_or_else(|| self.external_drop.allocate_token());

        let (position, kind, files) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                self.drain_effects(event_loop);
                return;
            };
            if state.external_drag_token.is_none() {
                state.external_drag_token = Some(token);
            }
            let position = fret_runner_winit::map_physical_position_to_point(
                state.window.scale_factor(),
                position,
            );
            state.external_drag_files = paths;
            let files = state.external_drag_files.clone();
            let kind =
                ExternalDragKind::EnterFiles(fret_runner_winit::external_drag_files(token, &files));
            (position, kind, files)
        };

        self.external_drop.set_payload_paths(token, files);

        self.deliver_window_event_now(
            app_window,
            &Event::ExternalDrag(ExternalDragEvent { position, kind }),
        );
        self.drain_effects(event_loop);
    }

    pub(super) fn handle_window_drag_moved(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        position: PhysicalPosition<f64>,
    ) {
        let (position, token) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                self.drain_effects(event_loop);
                return;
            };
            let position = fret_runner_winit::map_physical_position_to_point(
                state.window.scale_factor(),
                position,
            );
            (position, state.external_drag_token)
        };

        if let Some(token) = token {
            let paths = self.external_drop.paths(token).unwrap_or(&[]);
            let kind =
                ExternalDragKind::OverFiles(fret_runner_winit::external_drag_files(token, paths));
            self.deliver_window_event_now(
                app_window,
                &Event::ExternalDrag(ExternalDragEvent { position, kind }),
            );
        }
        self.drain_effects(event_loop);
    }

    pub(super) fn handle_window_drag_dropped(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        paths: Vec<PathBuf>,
        position: PhysicalPosition<f64>,
    ) {
        tracing::debug!(count = paths.len(), "winit drag dropped");
        let existing = self
            .windows
            .get(app_window)
            .and_then(|s| s.external_drag_token);
        let token = existing.unwrap_or_else(|| self.external_drop.allocate_token());

        let (position, kind, files) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                self.drain_effects(event_loop);
                return;
            };
            if state.external_drag_token.is_none() {
                state.external_drag_token = Some(token);
            }
            let position = fret_runner_winit::map_physical_position_to_point(
                state.window.scale_factor(),
                position,
            );
            if state.external_drag_files.is_empty() {
                state.external_drag_files = paths;
            }
            let files = std::mem::take(&mut state.external_drag_files);
            state.external_drag_token = None;
            let kind =
                ExternalDragKind::DropFiles(fret_runner_winit::external_drag_files(token, &files));
            (position, kind, files)
        };

        self.external_drop.set_payload_paths(token, files);

        self.deliver_window_event_now(
            app_window,
            &Event::ExternalDrag(ExternalDragEvent { position, kind }),
        );
        self.drain_effects(event_loop);
    }

    pub(super) fn handle_window_drag_left(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        position: Option<PhysicalPosition<f64>>,
    ) {
        tracing::debug!("winit drag left");
        let (position, token) = {
            let Some(state) = self.windows.get_mut(app_window) else {
                self.drain_effects(event_loop);
                return;
            };
            let position = fret_runner_winit::map_optional_physical_position_to_point(
                state.window.scale_factor(),
                position,
                state.platform.input.cursor_pos,
            );
            state.external_drag_files.clear();
            let token = state.external_drag_token.take();
            (position, token)
        };

        if let Some(token) = token {
            self.external_drop.release(token);
        }

        self.deliver_window_event_now(
            app_window,
            &Event::ExternalDrag(ExternalDragEvent {
                position,
                kind: ExternalDragKind::Leave,
            }),
        );
        self.drain_effects(event_loop);
    }
}
