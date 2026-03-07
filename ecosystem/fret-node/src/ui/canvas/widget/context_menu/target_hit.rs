use crate::core::{EdgeId, GroupId};
use crate::ui::canvas::widget::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in crate::ui::canvas::widget) fn hit_group_context_target<H: UiHost>(
        &self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        position: Point,
        zoom: f32,
    ) -> Option<GroupId> {
        let header_height = self.style.geometry.node_header_height;
        self.graph
            .read_ref(host, |graph| {
                let order =
                    crate::ui::canvas::geometry::group_order(graph, &snapshot.group_draw_order);
                for group_id in order.iter().rev() {
                    let Some(group) = graph.groups.get(group_id) else {
                        continue;
                    };

                    let rect0 = self.group_rect_with_preview(*group_id, group.rect);
                    let rect = group_resize::group_rect_to_px(rect0);
                    let handle = self.resize_handle_rect(rect, zoom);
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

    pub(in crate::ui::canvas::widget) fn hit_edge_context_target<H: UiHost>(
        &mut self,
        host: &mut H,
        snapshot: &ViewSnapshot,
        position: Point,
        zoom: f32,
    ) -> Option<EdgeId> {
        let (geometry, index) = self.canvas_derived(&*host, snapshot);
        self.graph
            .read_ref(host, |graph| {
                let mut scratch = HitTestScratch::default();
                let mut ctx =
                    HitTestCtx::new(geometry.as_ref(), index.as_ref(), zoom, &mut scratch);
                self.hit_edge(graph, snapshot, &mut ctx, position)
            })
            .ok()
            .flatten()
    }
}
