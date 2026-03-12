mod edge;
mod node;
mod resize;

use fret_core::{Modifiers, Point, Rect};
use fret_ui::UiHost;

use crate::core::NodeId as GraphNodeId;
use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};
use crate::ui::canvas::widget::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn handle_resize_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    node: GraphNodeId,
    rect: Rect,
    handle: NodeResizeHandle,
    zoom: f32,
) {
    resize::handle_resize_hit(canvas, cx, snapshot, position, node, rect, handle, zoom)
}

pub(super) fn handle_node_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    node: GraphNodeId,
    rect: Rect,
    multi_selection_pressed: bool,
    zoom: f32,
) {
    node::handle_node_hit(
        canvas,
        cx,
        snapshot,
        position,
        node,
        rect,
        multi_selection_pressed,
        zoom,
    )
}

pub(super) fn handle_edge_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: Modifiers,
    edge: crate::core::EdgeId,
    multi_selection_pressed: bool,
) {
    edge::handle_edge_hit(
        canvas,
        cx,
        snapshot,
        position,
        modifiers,
        edge,
        multi_selection_pressed,
    )
}
