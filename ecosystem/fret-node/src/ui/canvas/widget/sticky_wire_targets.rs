mod inspect;
mod picker;

use fret_core::Point;
use fret_ui::UiHost;

use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use crate::ui::canvas::state::ViewSnapshot;

pub(super) fn handle_sticky_wire_non_port_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    geom: &crate::ui::canvas::CanvasGeometry,
    index: &crate::ui::canvas::CanvasSpatialDerived,
    from_port: crate::core::PortId,
    position: Point,
    zoom: f32,
) -> bool {
    let at = canvas.interaction.last_canvas_pos.unwrap_or_default();
    let target =
        inspect::inspect_non_port_target(canvas, cx.app, snapshot, geom, index, position, zoom);

    reset_sticky_wire_state(canvas);
    cx.release_pointer_capture();

    match target {
        inspect::StickyWireNonPortTarget::Node => false,
        inspect::StickyWireNonPortTarget::Edge(edge_id) => {
            picker::open_edge_insert_node_picker(canvas, cx, edge_id, position)
        }
        inspect::StickyWireNonPortTarget::Canvas => {
            picker::open_connection_insert_node_picker(canvas, cx, from_port, at)
        }
    }
}

pub(super) fn reset_sticky_wire_state<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
) {
    canvas.interaction.sticky_wire = false;
    canvas.interaction.sticky_wire_ignore_next_up = false;
}
