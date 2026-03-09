use fret_core::PointerId;
use fret_ui::UiHost;

use super::searcher_activation::SearcherPointerHit;
use super::*;
use crate::ui::canvas::state::PendingInsertNodeDrag;

pub(super) fn clear_pending_searcher_row_drag(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    interaction.pending_insert_node_drag.take().is_some()
}

pub(super) fn dismiss_searcher_overlay<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) {
    canvas.interaction.searcher = None;
    clear_pending_searcher_row_drag(&mut canvas.interaction);
    cx.release_pointer_capture();
}

pub(super) fn sync_searcher_active_row_if_selectable<M: NodeGraphCanvasMiddleware>(
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

pub(super) fn arm_searcher_row_drag<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    row_ix: usize,
    position: Point,
) -> bool {
    if !sync_searcher_active_row_if_selectable(canvas, row_ix) {
        return false;
    }

    let Some(candidate) = canvas.interaction.searcher.as_ref().and_then(|searcher| {
        super::searcher_activation_hit::searcher_candidate_for_row(searcher, row_ix)
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

pub(super) fn activate_searcher_hit_or_dismiss<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    hit: SearcherPointerHit,
) {
    if let Some(row_ix) = hit.row_ix {
        let _ = canvas.try_activate_searcher_row(cx, row_ix);
    } else if !hit.inside {
        canvas.interaction.searcher = None;
    }
}

pub(super) fn finish_searcher_row_drag_release<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    hit: SearcherPointerHit,
) -> bool {
    if !clear_pending_searcher_row_drag(&mut canvas.interaction) {
        return false;
    }

    cx.release_pointer_capture();
    canvas.activate_searcher_hit_or_dismiss(cx, hit);
    super::searcher_ui::finish_searcher_event(cx)
}
