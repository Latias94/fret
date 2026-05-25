use fret_core::Modifiers;

use super::super::low_level_adapter::CanvasPaintInvalidationCx;
use super::super::searcher_ui::invalidate_searcher_paint;
use super::super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};
use fret_core::Point;

pub(super) fn handle_searcher_wheel_event<H, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasPaintInvalidationCx<H>,
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

#[cfg(test)]
mod tests;
