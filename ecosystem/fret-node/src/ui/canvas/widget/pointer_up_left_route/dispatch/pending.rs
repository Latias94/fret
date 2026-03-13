use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn handle_pending_release_chain<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    super::super::super::pointer_up_pending::handle_pending_group_drag_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_group_resize_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_node_drag_release(
            canvas, cx, snapshot, position, zoom,
        )
        || super::super::super::pointer_up_pending::handle_pending_node_resize_release(canvas, cx)
        || super::super::super::pointer_up_pending::handle_pending_wire_drag_release(
            canvas, cx, snapshot, position,
        )
}
