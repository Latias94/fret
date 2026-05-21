mod animation;
mod debounce;

use fret_ui::UiHost;

use super::*;

pub(super) fn handle_viewport_animation_tick<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    token: fret_core::TimerToken,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    animation::handle_viewport_animation_tick(canvas, cx, token)
}

pub(super) fn handle_viewport_move_debounce<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
    token: fret_core::TimerToken,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: viewport_motion_cx::ViewportMotionCx<H>,
{
    debounce::handle_viewport_move_debounce(canvas, cx, token)
}
