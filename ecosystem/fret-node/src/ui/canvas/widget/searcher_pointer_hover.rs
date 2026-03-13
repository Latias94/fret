mod state;

use super::*;

pub(super) use state::sync_searcher_hovered_row;

pub(super) fn update_searcher_hover_from_position<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    position: Point,
    zoom: f32,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    let hovered_row = super::hit_searcher_row(&canvas.style, searcher, position, zoom);
    sync_searcher_hovered_row::<M>(searcher, hovered_row)
}
