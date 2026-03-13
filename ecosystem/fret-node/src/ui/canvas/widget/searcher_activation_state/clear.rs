use fret_ui::UiHost;

use super::super::*;

pub(in super::super) fn clear_searcher_overlay(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    let mut cleared = false;
    if interaction.searcher.take().is_some() {
        cleared = true;
    }
    cleared |= clear_pending_searcher_row_drag(interaction);
    cleared
}

pub(in super::super) fn clear_pending_searcher_row_drag(
    interaction: &mut crate::ui::canvas::state::InteractionState,
) -> bool {
    interaction.pending_insert_node_drag.take().is_some()
}

pub(in super::super) fn dismiss_searcher_overlay<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) {
    clear_searcher_overlay(&mut canvas.interaction);
    cx.release_pointer_capture();
}

#[cfg(test)]
mod tests;
