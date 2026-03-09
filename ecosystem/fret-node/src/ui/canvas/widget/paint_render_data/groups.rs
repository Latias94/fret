use super::*;

impl<M: NodeGraphCanvasMiddleware> NodeGraphCanvasWith<M> {
    pub(in super::super) fn collect_group_render_data(
        &self,
        graph: &Graph,
        snapshot: &ViewSnapshot,
        selected_groups: &HashSet<crate::core::GroupId>,
        cull: Option<Rect>,
        out: &mut RenderData,
    ) {
        let order = group_order(graph, &snapshot.group_draw_order);
        out.metrics.group_total = order.len();
        for group_id in order {
            out.metrics.group_candidates = out.metrics.group_candidates.saturating_add(1);
            let Some(group) = graph.groups.get(&group_id) else {
                continue;
            };
            let rect0 = self.group_rect_with_preview(group_id, group.rect);
            let rect = Rect::new(
                Point::new(Px(rect0.origin.x), Px(rect0.origin.y)),
                Size::new(Px(rect0.size.width), Px(rect0.size.height)),
            );
            if cull.is_some_and(|c| !rects_intersect(rect, c)) {
                continue;
            }
            out.groups.push((
                rect,
                Arc::<str>::from(group.title.clone()),
                selected_groups.contains(&group_id),
            ));
            out.metrics.group_visible = out.metrics.group_visible.saturating_add(1);
        }
    }
}
