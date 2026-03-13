mod query;

use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use super::*;

pub(super) fn try_activate_active_searcher_row<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    let Some(row_ix) = canvas
        .interaction
        .searcher
        .as_ref()
        .map(|searcher| searcher.active_row)
    else {
        return false;
    };
    canvas.try_activate_searcher_row(cx, row_ix)
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
