use fret_core::{AppWindowId, Rect};
use fret_ui::EventCx;
use fret_ui::{CommandCx, UiHost};

use super::commit_cx::WireCommitCx;

impl<'a, H: UiHost> WireCommitCx<H> for EventCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, _last_bounds: Option<Rect>) -> Rect {
        self.bounds
    }
}

impl<'a, H: UiHost> WireCommitCx<H> for CommandCx<'a, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }

    fn bounds(&self, last_bounds: Option<Rect>) -> Rect {
        last_bounds.unwrap_or_default()
    }
}
