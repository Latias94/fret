use fret_ui::UiHost;

use super::super::searcher_activation::SearcherPointerHit;
use super::super::*;

pub(in super::super) fn activate_searcher_hit_or_dismiss<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    hit: SearcherPointerHit,
) {
    if let Some(row_ix) = hit.row_ix {
        let _ = canvas.try_activate_searcher_row(cx, row_ix);
    } else if !hit.inside {
        super::clear::clear_searcher_overlay(&mut canvas.interaction);
    }
}

pub(in super::super) fn finish_searcher_row_drag_release<
    H: UiHost,
    M: NodeGraphCanvasMiddleware,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    hit: SearcherPointerHit,
) -> bool {
    if !super::clear::clear_pending_searcher_row_drag(&mut canvas.interaction) {
        return false;
    }

    cx.release_pointer_capture();
    canvas.activate_searcher_hit_or_dismiss(cx, hit);
    super::super::searcher_ui::finish_searcher_event(cx)
}
