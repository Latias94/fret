use fret_core::Rect;
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::wire_drag_move_cx::WireDragMoveCx;

impl<H: UiHost> WireDragMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}
