use super::*;

pub(super) fn clear_expired_toast<H: UiHost, M: NodeGraphCanvasMiddleware>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut EventCx<'_, H>,
    token: fret_core::TimerToken,
) -> bool {
    if !canvas
        .interaction
        .toast
        .as_ref()
        .is_some_and(|toast| toast.timer == token)
    {
        return false;
    }

    canvas.interaction.toast = None;
    cx.request_redraw();
    cx.invalidate_self(Invalidation::Paint);
    true
}
