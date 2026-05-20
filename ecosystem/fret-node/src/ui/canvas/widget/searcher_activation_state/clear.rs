use crate::ui::canvas::state::InteractionState;

use super::super::{
    NodeGraphCanvasMiddleware, NodeGraphCanvasWith, widget_tail::PointerCaptureReleaseCx,
};

pub(in super::super) fn clear_searcher_overlay(interaction: &mut InteractionState) -> bool {
    let mut cleared = false;
    if interaction.searcher.take().is_some() {
        cleared = true;
    }
    cleared |= clear_pending_searcher_row_drag(interaction);
    cleared
}

pub(in super::super) fn clear_pending_searcher_row_drag(
    interaction: &mut InteractionState,
) -> bool {
    interaction.pending_insert_node_drag.take().is_some()
}

fn release_dismissed_searcher_capture<H>(cx: &mut impl PointerCaptureReleaseCx<H>) {
    cx.release_pointer_capture();
}

pub(in super::super) fn dismiss_searcher_overlay<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl PointerCaptureReleaseCx<H>,
) {
    clear_searcher_overlay(&mut canvas.interaction);
    release_dismissed_searcher_capture(cx);
}

#[cfg(test)]
mod tests;
