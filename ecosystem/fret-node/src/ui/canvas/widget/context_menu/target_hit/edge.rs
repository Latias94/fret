use crate::core::EdgeId;
use crate::ui::canvas::widget::*;

pub(super) fn hit_edge_context_target<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut H,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> Option<EdgeId> {
    let (geometry, index) = canvas.canvas_derived(&*host, snapshot);
    canvas
        .graph
        .read_ref(host, |graph| {
            let mut scratch = HitTestScratch::default();
            let mut ctx = HitTestCtx::new(geometry.as_ref(), index.as_ref(), zoom, &mut scratch);
            canvas.hit_edge(graph, snapshot, &mut ctx, position)
        })
        .ok()
        .flatten()
}
