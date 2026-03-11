use fret_core::PointerId;
use fret_ui::UiHost;

use super::super::*;
use crate::ui::canvas::state::PendingInsertNodeDrag;

pub(in super::super) fn sync_searcher_active_row_if_selectable<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    row_ix: usize,
) -> bool {
    let Some(searcher) = canvas.interaction.searcher.as_mut() else {
        return false;
    };
    if !searcher
        .rows
        .get(row_ix)
        .is_some_and(NodeGraphCanvasWith::<M>::searcher_is_selectable_row)
    {
        return false;
    }

    searcher.active_row = row_ix;
    NodeGraphCanvasWith::<M>::ensure_searcher_active_visible(searcher);
    true
}

pub(in super::super) fn arm_searcher_row_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    row_ix: usize,
    position: Point,
) -> bool {
    if !sync_searcher_active_row_if_selectable(canvas, row_ix) {
        return false;
    }

    let Some(candidate) = canvas.interaction.searcher.as_ref().and_then(|searcher| {
        super::super::searcher_activation_hit::searcher_candidate_for_row(searcher, row_ix)
    }) else {
        return false;
    };

    canvas.interaction.pending_insert_node_drag = Some(PendingInsertNodeDrag {
        candidate,
        start_pos: position,
        pointer_id: cx.pointer_id.unwrap_or(PointerId(0)),
        start_tick: cx.app.tick_id(),
    });
    cx.capture_pointer(cx.node);
    true
}
