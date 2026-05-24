use fret_core::AppWindowId;
use fret_ui::EventCx;
use fret_ui::UiHost;

use super::pointer_up_commit_cx::PointerUpCommitCx;

impl<H: UiHost> PointerUpCommitCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn window(&self) -> Option<AppWindowId> {
        self.window
    }
}
