use crate::ui::canvas::widget::*;

pub(super) fn apply_viewport_zoom<M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    host: &mut impl UiHost,
    position: Point,
    factor: f32,
) {
    canvas.zoom_about_pointer_factor(position, factor);
    let pan = canvas.cached_pan;
    let zoom = canvas.cached_zoom;
    canvas.update_view_state(host, |state| {
        state.pan = pan;
        state.zoom = zoom;
    });
}

pub(super) fn finish_viewport_zoom<H: UiHost>(cx: &mut EventCx<'_, H>) {
    super::super::paint_invalidation::invalidate_paint(cx);
}
