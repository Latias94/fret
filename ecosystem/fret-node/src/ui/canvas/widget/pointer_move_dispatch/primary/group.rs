use crate::ui::canvas::widget::*;

pub(super) fn dispatch_group_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    pending_group_drag::handle_pending_group_drag_move(canvas, cx, snapshot, position, zoom)
        || group_drag::handle_group_drag_move(canvas, cx, snapshot, position, modifiers, zoom)
        || pending_group_resize::handle_pending_group_resize_move(
            canvas, cx, snapshot, position, zoom,
        )
        || group_resize::handle_group_resize_move(canvas, cx, snapshot, position, modifiers, zoom)
}
