use fret_core::{Modifiers, Point};
use fret_ui::UiHost;

use super::connection_hits;
use super::element_hits;
use super::group_background;
use super::hit::Hit;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    zoom: f32,
    hit: Hit,
    multi_selection_pressed: bool,
) -> bool {
    match hit {
        Hit::Port(port) => {
            connection_hits::handle_port_hit(canvas, cx, snapshot, position, modifiers, zoom, port)
        }
        Hit::EdgeAnchor(edge, endpoint, fixed) => connection_hits::handle_edge_anchor_hit(
            canvas,
            cx,
            snapshot,
            position,
            edge,
            endpoint,
            fixed,
            multi_selection_pressed,
        ),
        Hit::Resize(node, rect, handle) => element_hits::handle_resize_hit(
            canvas, cx, snapshot, position, node, rect, handle, zoom,
        ),
        Hit::Node(node, rect) => element_hits::handle_node_hit(
            canvas,
            cx,
            snapshot,
            position,
            node,
            rect,
            multi_selection_pressed,
            zoom,
        ),
        Hit::Edge(edge) => element_hits::handle_edge_hit(
            canvas,
            cx,
            snapshot,
            position,
            modifiers,
            edge,
            multi_selection_pressed,
        ),
        Hit::GroupResize(group, rect) => group_background::handle_group_resize_hit(
            canvas,
            cx,
            snapshot,
            position,
            group,
            rect,
            multi_selection_pressed,
        ),
        Hit::GroupHeader(group, rect) => group_background::handle_group_header_hit(
            canvas,
            cx,
            snapshot,
            position,
            group,
            rect,
            multi_selection_pressed,
        ),
        Hit::Background => {
            group_background::handle_background_hit(canvas, cx, snapshot, position, modifiers)
        }
    }

    true
}
