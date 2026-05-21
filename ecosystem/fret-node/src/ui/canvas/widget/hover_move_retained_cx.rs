use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::hover_move_cx::HoverMoveCx;

impl<H: UiHost> HoverMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
