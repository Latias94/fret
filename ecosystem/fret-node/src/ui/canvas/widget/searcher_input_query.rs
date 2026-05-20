mod query;

use fret_core::{KeyCode, Modifiers};

use super::searcher_input::SearcherInputCx;
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn try_activate_active_searcher_row<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherInputCx<H, M>,
) -> bool {
    let Some(row_ix) = canvas
        .interaction
        .searcher
        .as_ref()
        .map(|searcher| searcher.active_row)
    else {
        return false;
    };
    cx.try_activate_searcher_row(canvas, row_ix)
}

pub(super) fn update_searcher_query_from_key<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    key: KeyCode,
    modifiers: Modifiers,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    if !query::apply_searcher_query_key(&mut searcher.query, key, modifiers) {
        return false;
    }

    NodeGraphCanvasWith::<M>::rebuild_searcher_rows(searcher);
    true
}
