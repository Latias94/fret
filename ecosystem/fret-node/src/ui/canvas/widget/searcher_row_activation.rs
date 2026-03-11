mod item;

use super::*;

pub(super) fn try_activate_searcher_row<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    row_ix: usize,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.take() else {
        return false;
    };

    let Some(item) = item::searcher_row_activation_item(&searcher, row_ix) else {
        canvas.interaction.searcher = Some(searcher);
        return false;
    };
    canvas.activate_context_menu_item(
        cx,
        &searcher.target,
        searcher.invoked_at,
        item,
        &searcher.candidates,
    );
    true
}
