mod step;

use super::searcher_input::SearcherStepDirection;
use super::*;

pub(super) use step::next_searcher_active_row;

pub(super) fn step_searcher_active_row<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    direction: SearcherStepDirection,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    let Some(next_ix) = next_searcher_active_row(&searcher.rows, searcher.active_row, direction)
    else {
        return false;
    };

    searcher.active_row = next_ix;
    NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    true
}
