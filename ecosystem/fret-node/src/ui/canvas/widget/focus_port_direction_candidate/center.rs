use crate::core::CanvasPoint;
use crate::ui::canvas::geometry::PortHandleGeometry;

pub(super) fn handle_center(handle: &PortHandleGeometry) -> CanvasPoint {
    CanvasPoint {
        x: handle.center.x.0,
        y: handle.center.y.0,
    }
}
