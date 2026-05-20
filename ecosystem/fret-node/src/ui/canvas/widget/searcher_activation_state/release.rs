use super::super::searcher_activation::SearcherPointerHit;
use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, widget_tail::HandledPointerCaptureReleaseCx,
};

pub(in super::super) trait SearcherReleaseCx<H, M: NodeGraphCanvasMiddleware>:
    HandledPointerCaptureReleaseCx<H>
{
    fn try_activate_searcher_row(
        &mut self,
        canvas: &mut NodeGraphCanvasWith<M>,
        row_ix: usize,
    ) -> bool;
}

pub(in super::super) fn activate_searcher_hit_or_dismiss<
    H,
    M: NodeGraphCanvasMiddleware,
    Cx: SearcherReleaseCx<H, M>,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    hit: SearcherPointerHit,
) {
    if let Some(row_ix) = hit.row_ix {
        let _ = cx.try_activate_searcher_row(canvas, row_ix);
    } else if !hit.inside {
        super::clear::clear_searcher_overlay(&mut canvas.interaction);
    }
}

pub(in super::super) fn finish_searcher_row_drag_release<
    H,
    M: NodeGraphCanvasMiddleware,
    Cx: SearcherReleaseCx<H, M>,
>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    hit: SearcherPointerHit,
) -> bool {
    if !super::clear::clear_pending_searcher_row_drag(&mut canvas.interaction) {
        return false;
    }

    cx.release_pointer_capture();
    activate_searcher_hit_or_dismiss(canvas, cx, hit);
    super::super::searcher_ui::finish_searcher_event(cx)
}

#[cfg(test)]
mod tests;
