use fret_core::Rect;
use fret_ui::{UiHost, retained_bridge::EventCx};

use super::ContextMenuOpeningCx;

impl<H: UiHost> ContextMenuOpeningCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn has_window(&self) -> bool {
        self.window.is_some()
    }
}
