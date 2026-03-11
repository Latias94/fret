use fret_core::Modifiers;
use fret_ui::UiHost;

use super::super::searcher_ui::invalidate_searcher_paint;
use super::super::*;

pub(super) fn handle_searcher_wheel_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    delta: Point,
    modifiers: Modifiers,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.scroll_searcher_from_wheel(delta, modifiers) {
        invalidate_searcher_paint(cx);
        return true;
    }

    !modifiers.ctrl && !modifiers.meta
}
