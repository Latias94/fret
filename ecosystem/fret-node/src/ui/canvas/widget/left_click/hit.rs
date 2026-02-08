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
            if let Some(port) = this.hit_port(&mut ctx, position) {
                return Hit::Port(port);
            }

            if let Some((edge, endpoint, fixed)) =
                this.hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
            {
                return Hit::EdgeAnchor(edge, endpoint, fixed);
            }

            let order = geom.order.clone();
            let Some(node) = order.iter().rev().find_map(|id| {
                geom.nodes
                    .get(id)
                    .is_some_and(|ng| ng.rect.contains(position))
                    .then_some(*id)
            }) else {
                if let Some(edge) = this.hit_edge(graph, snapshot, &mut ctx, position) {
                    return Hit::Edge(edge);
                }

                let order =
                    crate::ui::canvas::geometry::group_order(graph, &snapshot.group_draw_order);
                for group_id in order.iter().rev() {
                    let Some(group) = graph.groups.get(group_id) else {
                        continue;
                    };
                    let rect0 = this.group_rect_with_preview(*group_id, group.rect);
                    let rect = crate::ui::canvas::widget::group_resize::group_rect_to_px(rect0);
                    let handle = this.resize_handle_rect(rect, zoom);
                    if crate::ui::canvas::widget::group_resize::group_resize_handle_hit(
                        handle, position, zoom, 6.0,
                    ) {
                        return Hit::GroupResize(*group_id, rect0);
                    }
                }

                let header_h = this.style.node_header_height;
                for group_id in order.iter().rev() {
                    let Some(group) = graph.groups.get(group_id) else {
                        continue;
                    };
                    let rect0 = this.group_rect_with_preview(*group_id, group.rect);
                    if !crate::ui::canvas::widget::pending_group_drag::group_header_hit(
                        rect0, header_h, zoom, position,
                    ) {
                        continue;
                    }
                    return Hit::GroupHeader(*group_id, rect0);
                }
                return Hit::Background;
            };
            let Some(rect) = geom.nodes.get(&node).map(|ng| ng.rect) else {
                return Hit::Background;
            };
            let is_selected = snapshot.selected_nodes.iter().any(|id| *id == node);
            if is_selected {
                let resize_handles = this.presenter.node_resize_handles(graph, node, &this.style);
                for handle in NodeResizeHandle::ALL {
                    if !resize_handles.contains(handle) {
                        continue;
                    }
                    let hit_rect = this.node_resize_handle_rect(rect, handle, zoom);
                    if NodeGraphCanvasWith::<M>::rect_contains(hit_rect, position) {
                        return Hit::Resize(node, rect, handle);
                    }
                }
            }

            Hit::Node(node, rect)
        })
        .unwrap_or(Hit::Background)
}
