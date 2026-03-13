use fret_core::Point;
use fret_ui::UiHost;

use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::*;

pub(super) fn handle_active_release_chain<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    super::super::super::wire_drag::handle_wire_left_up(canvas, cx, snapshot, zoom)
        || super::super::super::edge_insert_drag::handle_edge_insert_left_up(canvas, cx, position)
        || super::super::super::edge_drag::handle_edge_left_up(canvas, cx)
}
