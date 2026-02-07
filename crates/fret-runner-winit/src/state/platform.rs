use fret_core::{CursorIcon, Event, Rect};
use winit::event::WindowEvent;
use winit::window::Window;

use crate::mapping::WheelConfig;

use super::input::WinitInputState;
use super::window::WinitWindowState;

#[derive(Debug, Default, Clone)]
pub struct WinitPlatform {
    pub input: WinitInputState,
    pub wheel: WheelConfig,
    pub window: WinitWindowState,
}

impl WinitPlatform {
    pub fn handle_window_event(
        &mut self,
        window_scale_factor: f64,
        event: &WindowEvent,
        out: &mut Vec<Event>,
    ) {
        self.input
            .handle_window_event_with_config(window_scale_factor, event, self.wheel, out);
    }

    pub fn set_ime_allowed(&mut self, enabled: bool) -> bool {
        self.window.set_ime_allowed(enabled)
    }

    pub fn set_ime_cursor_area(&mut self, rect: Rect) -> bool {
        self.window.set_ime_cursor_area(rect)
    }

    pub fn ime_cursor_area(&self) -> Option<Rect> {
        self.window.ime_cursor_area()
    }

    pub fn set_cursor_icon(&mut self, icon: CursorIcon) -> bool {
        self.window.set_cursor_icon(icon)
    }

    /// Applies any pending window-side state (IME/cursor) before drawing a frame.
    ///
    /// This mirrors the backend split pattern in Dear ImGui (`prepare_frame`).
    pub fn prepare_frame(&mut self, window: &dyn Window) {
        self.window.prepare_frame(window);
    }
}
