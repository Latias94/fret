mod background;
mod connection;
mod node;

use fret_core::{Point, Rect};
use fret_ui::UiHost;

use crate::core::{EdgeId, GroupId, NodeId as GraphNodeId, PortId};
use crate::rules::EdgeEndpoint;

use crate::ui::canvas::state::{NodeResizeHandle, ViewSnapshot};
use crate::ui::canvas::widget::{
    HitTestCtx, HitTestScratch, NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
};

#[derive(Debug, Clone, Copy)]
pub(super) enum Hit {
    Port(PortId),
    EdgeAnchor(EdgeId, EdgeEndpoint, PortId),
    Resize(GraphNodeId, Rect, NodeResizeHandle),
    Node(GraphNodeId, Rect),
    Edge(EdgeId),
    GroupResize(GroupId, crate::core::CanvasRect),
    GroupHeader(GroupId, crate::core::CanvasRect),
    Background,
}

pub(super) fn compute_hit<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Hit {
    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    let this = &*canvas;
    this.graph
        .read_ref(cx.app, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);
            if let Some(hit) =
                connection::compute_connection_hit(this, graph, snapshot, &mut ctx, position)
            {
                return hit;
            }

            let order = geom.order.clone();
            let Some(node) = order.iter().rev().find_map(|id| {
                geom.nodes
                    .get(id)
                    .is_some_and(|ng| ng.rect.contains(position))
                    .then_some(*id)
            }) else {
                return background::compute_background_hit(
                    this, graph, snapshot, &mut ctx, position, zoom,
                );
            };
            let Some(rect) = geom.nodes.get(&node).map(|ng| ng.rect) else {
                return Hit::Background;
            };
            node::compute_node_hit::<M>(this, graph, snapshot, node, rect, position, zoom)
        })
        .unwrap_or(Hit::Background)
}
