use crate::ui::canvas::widget::*;

pub(super) fn dispatch_surface_move_handlers<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    pan_zoom::handle_panning_move(canvas, cx, snapshot, position)
        || marquee::handle_marquee_move(canvas, cx, snapshot, position, modifiers, zoom)
}
