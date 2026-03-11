use super::super::*;

pub(super) fn handle_searcher_pointer_up_event<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    position: Point,
    button: MouseButton,
    zoom: f32,
) -> bool {
    if button != MouseButton::Left {
        return false;
    }
    if canvas.interaction.searcher.is_none() {
        super::super::searcher_activation_state::clear_pending_searcher_row_drag(
            &mut canvas.interaction,
        );
        return false;
    }

    let hit = super::super::searcher_activation_hit::searcher_pointer_hit(canvas, position, zoom);
    super::super::searcher_activation_state::finish_searcher_row_drag_release(canvas, cx, hit)
}
