use fret_core::AppWindowId;
use fret_ui::{EventCx, UiHost};

use super::BackgroundInsertMenuCx;

impl<H: UiHost> BackgroundInsertMenuCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}
