use fret_core::{AppWindowId, Rect};
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::auto_pan_timer_cx::AutoPanTimerCx;

impl<H: UiHost> AutoPanTimerCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}
