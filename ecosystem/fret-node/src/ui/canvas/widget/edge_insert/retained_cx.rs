use fret_core::{AppWindowId, Rect};
use fret_ui::{EventCx, UiHost};

use super::cx::EdgeInsertCx;

impl<H: UiHost> EdgeInsertCx<H> for EventCx<'_, H> {
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
