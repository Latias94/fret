use fret_ui::EventCx;
use fret_ui::UiHost;

use super::node_drag_preview_cx::NodeDragPreviewCx;

impl<H: UiHost> NodeDragPreviewCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
