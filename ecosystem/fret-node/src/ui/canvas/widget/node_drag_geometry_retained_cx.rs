use fret_ui::EventCx;
use fret_ui::UiHost;

use super::node_drag_geometry_cx::NodeDragGeometryCx;

impl<H: UiHost> NodeDragGeometryCx<H> for EventCx<'_, H> {
    fn host(&mut self) -> &mut H {
        self.app
    }
}
