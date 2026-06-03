use fret_core::time::Instant;
use fret_core::{AppWindowId, Size, WindowLogicalPosition};
use fret_runtime::WindowResizeDirection;

use super::window::bring_window_to_front;
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn apply_window_visibility_request(&mut self, window: AppWindowId, visible: bool) {
        if let Some(state) = self.windows.get(window) {
            state.window.set_visible(visible);
            state.window.request_redraw();
        }
    }

    pub(super) fn apply_window_inner_size_request(&mut self, window: AppWindowId, size: Size) {
        let applied = if let Some(state) = self.windows.get_mut(window) {
            let requested = winit::dpi::LogicalSize::new(size.width.0 as f64, size.height.0 as f64);
            Some(
                state
                    .window
                    .request_surface_size(requested.into())
                    // Some platforms apply the resize without emitting a resize event and return
                    // `None` here. Fall back to the current surface size so diagnostics converge.
                    .unwrap_or_else(|| state.window.surface_size()),
            )
        } else {
            None
        };

        if let Some(applied) = applied {
            // Match the OS resize-event path: sync the surface immediately, but keep
            // window-metrics delivery coalesced to the next redraw.
            self.sync_surface_resize_now(window, applied);
            self.request_surface_resize_redraw(window);
        }
    }

    pub(super) fn apply_window_outer_position_request(
        &mut self,
        window: AppWindowId,
        position: WindowLogicalPosition,
    ) {
        if let Some(state) = self.windows.get(window) {
            #[cfg(target_os = "windows")]
            {
                // On Windows, winit's `Position::Logical` is monitor-local. Convert to absolute
                // physical pixels for deterministic scripted placement across multi-monitor setups.
                let scale = state.window.scale_factor().max(0.000_001);
                let x = (position.x as f64 * scale).round() as i32;
                let y = (position.y as f64 * scale).round() as i32;
                if let Some(hwnd) = Self::hwnd_for_window(state.window.as_ref()) {
                    let _ = super::win32::set_window_outer_position(hwnd, x, y);
                } else {
                    state
                        .window
                        .set_outer_position(winit::dpi::Position::Physical(
                            winit::dpi::PhysicalPosition::new(x, y),
                        ));
                }
            }
            #[cfg(not(target_os = "windows"))]
            state
                .window
                .set_outer_position(winit::dpi::Position::Logical(
                    winit::dpi::LogicalPosition::new(position.x as f64, position.y as f64),
                ));
            state.window.request_redraw();
        }
    }

    pub(super) fn apply_window_raise_request(
        &mut self,
        window: AppWindowId,
        sender_id: Option<AppWindowId>,
        _now: Instant,
    ) {
        let sender_window = sender_id
            .and_then(|id| self.windows.get(id))
            .map(|w| w.window.as_ref());
        if let Some(state) = self.windows.get(window) {
            let _ = bring_window_to_front(state.window.as_ref(), sender_window);
            state.window.request_redraw();
        }
        #[cfg(target_os = "macos")]
        {
            if self.windows.contains_key(window) {
                self.enqueue_window_front(window, sender_id, None, _now);
            }
        }
    }

    pub(super) fn begin_window_drag_request(&mut self, window: AppWindowId) {
        if let Some(state) = self.windows.get(window) {
            let _ = state.window.drag_window();
        }
    }

    pub(super) fn begin_window_resize_request(
        &mut self,
        window: AppWindowId,
        direction: WindowResizeDirection,
    ) {
        if let Some(state) = self.windows.get(window) {
            let direction = match direction {
                WindowResizeDirection::N => winit::window::ResizeDirection::North,
                WindowResizeDirection::Ne => winit::window::ResizeDirection::NorthEast,
                WindowResizeDirection::E => winit::window::ResizeDirection::East,
                WindowResizeDirection::Se => winit::window::ResizeDirection::SouthEast,
                WindowResizeDirection::S => winit::window::ResizeDirection::South,
                WindowResizeDirection::Sw => winit::window::ResizeDirection::SouthWest,
                WindowResizeDirection::W => winit::window::ResizeDirection::West,
                WindowResizeDirection::Nw => winit::window::ResizeDirection::NorthWest,
            };
            let _ = state.window.drag_resize_window(direction);
        }
    }
}
