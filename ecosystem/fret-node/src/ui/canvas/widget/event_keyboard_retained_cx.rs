use fret_ui::{EventCx, UiHost};

use super::event_keyboard::KeyboardInputFocusCx;

impl<H: UiHost> KeyboardInputFocusCx for EventCx<'_, H> {
    fn focus_is_text_input(&self) -> bool {
        self.input_ctx.focus_is_text_input
    }
}
