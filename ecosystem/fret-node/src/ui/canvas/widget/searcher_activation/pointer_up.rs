use fret_core::{MouseButton, Point};

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    searcher_activation_state::{
        SearcherReleaseCx, clear_pending_searcher_row_drag, finish_searcher_row_drag_release,
    },
};
use super::SearcherPointerHit;

pub(super) fn handle_searcher_pointer_up_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherReleaseCx<H, M>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }
    if canvas.interaction.searcher.is_none() {
        clear_pending_searcher_row_drag(&mut canvas.interaction);
        return false;
    }

    let hit = super::super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
    handle_searcher_pointer_up_hit(canvas, cx, hit)
}

fn handle_searcher_pointer_up_hit<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherReleaseCx<H, M>,
    hit: SearcherPointerHit,
) -> bool {
    finish_searcher_row_drag_release(canvas, cx, hit)
}

#[cfg(test)]
mod tests;
