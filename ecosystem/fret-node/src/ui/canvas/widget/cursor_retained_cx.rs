use fret_core::CursorIcon;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::cursor_cx::CanvasCursorCx;

impl<H: UiHost> CanvasCursorCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn set_cursor_icon(&mut self, icon: CursorIcon) {
        self.set_cursor_icon(icon);
    }
}
