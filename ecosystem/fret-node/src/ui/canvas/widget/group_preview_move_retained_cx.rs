use fret_core::Rect;
use fret_ui::UiHost;
use fret_ui::retained_bridge::EventCx;

use super::group_preview_move_cx::GroupPreviewMoveCx;

impl<H: UiHost> GroupPreviewMoveCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }
}
