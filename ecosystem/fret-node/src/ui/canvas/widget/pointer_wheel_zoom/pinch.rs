use crate::ui::canvas::widget::*;

use super::apply;

pub(super) fn handle_pinch_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: f32,
) -> bool {
    if !snapshot.interaction.zoom_on_pinch || !delta.is_finite() {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::ZoomPinch);
    let speed = snapshot.interaction.zoom_on_pinch_speed.max(0.0);
    let factor = (1.0 + delta.clamp(-0.95, 10.0) * speed).max(0.01);
    apply::apply_viewport_zoom(canvas, cx.app, position, factor);
    apply::finish_viewport_zoom(cx);
    true
}
