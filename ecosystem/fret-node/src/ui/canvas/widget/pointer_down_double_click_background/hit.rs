use super::super::*;

pub(super) fn pointer_is_background<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    let (geom, index) = canvas.canvas_derived(&*cx.app, snapshot);
    canvas
        .graph
        .read_ref(cx.app, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geom.as_ref(), index.as_ref(), zoom, &mut scratch);

            if canvas.hit_port(&mut ctx, position).is_some() {
                return false;
            }
            if canvas
                .hit_edge_focus_anchor(graph, snapshot, &mut ctx, position)
                .is_some()
            {
                return false;
            }
            if geom.nodes.values().any(|ng| ng.rect.contains(position)) {
                return false;
            }
            if canvas
                .hit_edge(graph, snapshot, &mut ctx, position)
                .is_some()
            {
                return false;
            }
            !graph.groups.iter().any(|(group_id, group)| {
                let rect0 = canvas.group_rect_with_preview(*group_id, group.rect);
                group_resize::group_rect_to_px(rect0).contains(position)
            })
        })
        .unwrap_or(false)
}
