use crate::ui::canvas::widget::*;

pub(super) fn dispatch_overlay_pointer_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    zoom: f32,
) -> bool {
    searcher::handle_searcher_pointer_move(canvas, cx, position, zoom)
        || context_menu::handle_context_menu_pointer_move(canvas, cx, position, zoom)
        || {
            let _ = snapshot;
            false
        }
}
