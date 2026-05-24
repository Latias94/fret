use fret_core::Rect;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::pan_zoom_begin_cx::{PanZoomBeginCx, PanZoomCx};

impl<H: UiHost> PanZoomCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}

impl<H: UiHost> PanZoomBeginCx<H> for EventCx<'_, H> {
    fn capture_self_pointer(&mut self) {
        self.capture_pointer(self.node);
    }
}
