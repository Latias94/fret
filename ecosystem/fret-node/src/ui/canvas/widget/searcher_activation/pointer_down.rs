use fret_core::{MouseButton, Point};

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith,
    searcher_activation_state::SearcherArmCx,
    searcher_ui::{dismiss_searcher_event, finish_searcher_event},
    widget_tail::HandledPointerCaptureReleaseCx,
};
use super::SearcherPointerHit;

pub(in super::super) trait SearcherPointerDownCx<H>:
    SearcherArmCx + HandledPointerCaptureReleaseCx<H>
{
}

impl<H, T> SearcherPointerDownCx<H> for T where T: SearcherArmCx + HandledPointerCaptureReleaseCx<H> {}

pub(super) fn handle_searcher_pointer_down_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherPointerDownCx<H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    let hit = super::super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
    handle_searcher_pointer_down_hit(canvas, cx, position, button, hit)
}

fn handle_searcher_pointer_down_hit<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl SearcherPointerDownCx<H>,
    position: Point,
    button: MouseButton,
    hit: SearcherPointerHit,
) -> bool {
    match button {
        MouseButton::Left => {
            if let Some(row_ix) = hit.row_ix {
                let _ = canvas.arm_searcher_row_drag(cx, row_ix, position);
            } else if !hit.inside {
                canvas.dismiss_searcher_overlay(cx);
            }
            finish_searcher_event(cx)
        }
        _ => dismiss_searcher_event(canvas, cx),
    }
}

#[cfg(test)]
mod tests;
