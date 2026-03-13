use fret_ui::UiHost;

use super::super::searcher_ui::invalidate_searcher_paint;
use super::super::*;

pub(super) fn handle_searcher_pointer_move_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    zoom: f32,
) -> bool {
    if canvas.interaction.searcher.is_none() {
        return false;
    }

    if canvas.update_searcher_hover_from_position(position, zoom) {
        invalidate_searcher_paint(cx);
    }
    true
}
