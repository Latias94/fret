mod animation;
mod debounce;

use fret_ui::UiHost;

use super::*;

pub(super) fn handle_viewport_animation_tick<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    animation::handle_viewport_animation_tick(canvas, cx, token)
}

pub(super) fn handle_viewport_move_debounce<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    debounce::handle_viewport_move_debounce(canvas, cx, token)
}
