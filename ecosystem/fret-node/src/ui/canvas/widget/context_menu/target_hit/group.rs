use crate::core::GroupId;
use crate::ui::canvas::widget::*;

pub(super) fn hit_group_context_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<GroupId> {
    let header_height = canvas.style.geometry.node_header_height;
    canvas
        .graph
        .read_ref(host, |graph| {
            let order = crate::ui::canvas::geometry::group_order(graph, &snapshot.group_draw_order);
            for group_id in order.iter().rev() {
                let Some(group) = graph.groups.get(group_id) else {
                    continue;
                };

                let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
                let rect = group_resize::group_rect_to_px(rect0);
                let handle = canvas.resize_handle_rect(rect, zoom);
                if group_resize::group_resize_handle_hit(handle, position, zoom, 6.0) {
                    return Some(*group_id);
                }

                if pending_group_drag::group_header_hit(rect0, header_height, zoom, position) {
                    return Some(*group_id);
                }
            }
            None
        })
        .ok()
        .flatten()
}
