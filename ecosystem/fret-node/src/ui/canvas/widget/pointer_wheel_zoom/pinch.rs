use crate::ui::canvas::widget::*;

use super::apply;

pub(super) fn handle_pinch_zoom<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    if !snapshot.interaction.zoom_on_pinch || !delta.is_finite() {
        return false;
    }

    let window = cx.window();
    canvas.bump_viewport_move_debounce(cx.host(), window, snapshot, ViewportMoveKind::ZoomPinch);
    let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
    let factor = (1.0 + delta.clamp(-0.95, 10.0) * speed).max(0.01);
    apply::apply_viewport_zoom(canvas, cx.host(), position, factor);
    apply::finish_viewport_zoom(cx);
    true
}
