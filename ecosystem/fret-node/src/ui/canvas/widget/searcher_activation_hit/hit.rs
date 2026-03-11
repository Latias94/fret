use super::super::searcher_activation::SearcherPointerHit;
use super::super::*;

pub(in super::super) fn searcher_pointer_hit<M: NodeGraphCanvasMiddleware>(
    canvas: &NodeGraphCanvasWith<M>,
    position: Point,
    zoom: f32,
) -> SearcherPointerHit {
    let Some(searcher) = canvas.interaction.searcher.as_ref() else {
        return SearcherPointerHit::default();
    };

    let visible = super::super::searcher_visible_rows(searcher);
    let rect = super::super::searcher_rect_at(&canvas.style, searcher.origin, visible, zoom);
    SearcherPointerHit {
        inside: rect.contains(position),
        row_ix: super::super::hit_searcher_row(&canvas.style, searcher, position, zoom),
    }
}
