use crate::ui::canvas::widget::*;

use super::apply;

pub(super) fn handle_scroll_zoom<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    snapshot: &ViewSnapshot,
    position: Point,
    delta: Point,
    modifiers: fret_core::Modifiers,
    zoom: f32,
) -> bool {
    let zoom_active = snapshot
        .interaction
        .zoom_activation_key
        .is_pressed(modifiers);
    if !(snapshot.interaction.zoom_on_scroll && zoom_active) {
        return false;
    }

    canvas.bump_viewport_move_debounce(cx.app, cx.window, snapshot, ViewportMoveKind::ZoomWheel);
    let speed = snapshot.interaction.zoom_on_scroll_speed.max(0.0);
    let delta_screen_y = delta.y.0 * zoom;
    let factor = fret_canvas::view::wheel_zoom_factor(
        delta_screen_y,
        fret_canvas::view::DEFAULT_WHEEL_ZOOM_BASE,
        fret_canvas::view::DEFAULT_WHEEL_ZOOM_STEP,
        speed,
    )
    .unwrap_or(1.0);
    apply::apply_viewport_zoom(canvas, cx.app, position, factor);
    apply::finish_viewport_zoom(cx);
    true
}
