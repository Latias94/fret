//! Winit `ApplicationHandler` integration.

use super::*;

use fret_core::time::Instant;

impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {
    fn device_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        _device_id: Option<winit::event::DeviceId>,
        event: DeviceEvent,
    ) {
        self.handle_device_event(event_loop, event);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_can_create_surfaces(event_loop);
    }

    fn destroy_surfaces(&mut self, _event_loop: &dyn ActiveEventLoop) {
        self.handle_destroy_surfaces();
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_proxy_wake_up(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(app_window) = self.window_registry.get(window_id) else {
            return;
        };

        self.handle_window_pre_dispatch_event(app_window, &event);

        match event {
            #[cfg(all(target_os = "macos", feature = "macos-hit-test-regions"))]
            WindowEvent::Moved(..) => {
                self.handle_window_moved();
            }
            ref ev @ WindowEvent::ModifiersChanged(..) => {
                self.handle_window_modifiers_changed(event_loop, app_window, ev);
            }
            WindowEvent::ThemeChanged(_theme) => {
                self.handle_window_theme_changed(app_window);
            }
            WindowEvent::Focused(focused) => {
                self.handle_window_focus_changed(app_window, window_id, focused);
            }
            WindowEvent::DragEntered { paths, position } => {
                self.handle_window_drag_entered(event_loop, app_window, paths, position);
            }
            WindowEvent::DragMoved { position } => {
                self.handle_window_drag_moved(event_loop, app_window, position);
            }
            WindowEvent::DragDropped { paths, position } => {
                self.handle_window_drag_dropped(event_loop, app_window, paths, position);
            }
            WindowEvent::DragLeft { position } => {
                self.handle_window_drag_left(event_loop, app_window, position);
            }
            WindowEvent::SurfaceResized(size) => {
                self.handle_window_surface_resized(event_loop, app_window, size);
            }
            ref ev @ WindowEvent::PointerMoved { .. } => {
                self.handle_window_pointer_moved(event_loop, app_window, ev);
            }
            ref ev @ WindowEvent::PointerButton { .. } => {
                self.handle_window_pointer_button(event_loop, app_window, ev);
            }
            WindowEvent::RedrawRequested => {
                self.handle_window_redraw_requested(event_loop, app_window);
            }
            ref ev => {
                self.handle_window_mapped_event(event_loop, app_window, ev);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.handle_about_to_wait_preamble(event_loop) {
            return;
        }

        #[cfg(feature = "diag-screenshots")]
        self.handle_about_to_wait_diag_screenshots();

        #[cfg(any(target_os = "android", target_os = "ios"))]
        self.handle_about_to_wait_mobile_surface_recreation(event_loop);

        self.handle_about_to_wait_internal_drag_poll(event_loop);
        let turn_now = self.handle_about_to_wait_turn_bookkeeping();

        #[cfg(all(
            feature = "dev-state",
            not(any(target_os = "android", target_os = "ios"))
        ))]
        self.handle_about_to_wait_dev_state_observation(turn_now);
        #[cfg(not(all(
            feature = "dev-state",
            not(any(target_os = "android", target_os = "ios"))
        )))]
        let _ = turn_now;

        self.handle_about_to_wait_window_platform_and_accessibility();

        self.handle_about_to_wait_dock_follow_stop();

        self.drain_effects(event_loop);

        let now = Instant::now();
        self.handle_about_to_wait_dock_released_outside_fallbacks(event_loop);

        self.handle_about_to_wait_control_flow(event_loop, now);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_resumed(event_loop);
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.handle_suspended(event_loop);
    }
}
