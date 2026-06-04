use fret_core::{AppWindowId, Event};
use winit::{event::WindowEvent, event_loop::ActiveEventLoop, window::WindowId};

use super::{WinitAppDriver, WinitRunner, macos_window_log};

impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn handle_window_modifiers_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        app_window: AppWindowId,
        ev: &WindowEvent,
    ) {
        if let Some(state) = self.windows.get_mut(app_window) {
            state
                .platform
                .handle_window_event(state.window.scale_factor(), ev, &mut Vec::new());
        }

        if self.internal_drag_routing_pointer_id().is_some() {
            self.route_internal_drag_hover_from_cursor();
            self.drain_effects(event_loop);
        }
    }

    pub(super) fn handle_window_theme_changed(&mut self, app_window: AppWindowId) {
        let window_ref = self.windows.get(app_window).map(|s| s.window.clone());
        if let Some(window_ref) = window_ref
            && self.update_window_environment_for_window_ref(app_window, window_ref.as_ref())
        {
            self.app.request_redraw(app_window);
        }
    }

    pub(super) fn handle_window_focus_changed(
        &mut self,
        app_window: AppWindowId,
        window_id: WindowId,
        focused: bool,
    ) {
        if let Some(state) = self.windows.get_mut(app_window) {
            state.is_focused = focused;
            if !focused {
                state.platform.input.pressed_buttons = fret_core::MouseButtons::default();
            }
        }
        if focused {
            self.bump_window_z_order(app_window);
        }
        self.deliver_window_event_now(app_window, &Event::WindowFocusChanged(focused));
        macos_window_log(format_args!(
            "[focused] app_window={:?} focused={} winit={:?}",
            app_window, focused, window_id
        ));
    }
}
