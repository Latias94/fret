use fret_core::Point;

use crate::core::Graph;
use crate::ui::canvas::state::ViewSnapshot;
use crate::ui::canvas::widget::{HitTestCtx, NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

use super::Hit;

pub(super) fn compute_background_hit<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    graph: &Graph,
    snapshot: &ViewSnapshot,
    ctx: &mut HitTestCtx<'_>,
    position: Point,
    zoom: f32,
) -> Hit {
    if let Some(edge) = canvas.hit_edge(graph, snapshot, ctx, position) {
        return Hit::Edge(edge);
    }

    let order = crate::ui::canvas::geometry::group_order(graph, &snapshot.group_draw_order);
    for group_id in order.iter().rev() {
        let Some(group) = graph.groups.get(group_id) else {
            continue;
        };
        let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
        let rect = crate::ui::canvas::widget::group_resize::group_rect_to_px(rect0);
        let handle = canvas.resize_handle_rect(rect, zoom);
        if crate::ui::canvas::widget::group_resize::group_resize_handle_hit(
            handle, position, zoom, 6.0,
        ) {
            return Hit::GroupResize(*group_id, rect0);
        }
    }

    let header_h = canvas.style.geometry.node_header_height;
    for group_id in order.iter().rev() {
        let Some(group) = graph.groups.get(group_id) else {
            continue;
        };
        let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
        if !crate::ui::canvas::widget::pending_group_drag::group_header_hit(
            rect0, header_h, zoom, position,
        ) {
            continue;
        }
        return Hit::GroupHeader(*group_id, rect0);
    }

    Hit::Background
}
