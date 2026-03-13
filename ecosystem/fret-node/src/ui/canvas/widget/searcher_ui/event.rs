use fret_ui::UiHost;

use super::super::*;

pub(super) fn dismiss_searcher_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    canvas.dismiss_searcher_overlay(cx);
    finish_searcher_event(cx)
}

pub(super) fn handle_searcher_escape_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
) -> bool {
    dismiss_searcher_event(canvas, cx)
}

pub(super) fn invalidate_searcher_paint<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) {
    super::super::paint_invalidation::invalidate_paint(cx);
}

pub(super) fn finish_searcher_event<H: UiHost>(
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    cx.stop_propagation();
    invalidate_searcher_paint(cx);
    true
}
