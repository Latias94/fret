use fret_core::AppWindowId;
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::picker::StickyWireTargetPickerCx;

impl<H: UiHost> StickyWireTargetPickerCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}
