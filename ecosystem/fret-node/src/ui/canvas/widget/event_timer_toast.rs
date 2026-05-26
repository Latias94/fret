use fret_core::TimerToken;
use fret_ui::UiHost;

use super::low_level_adapter::{CanvasPaintInvalidationCx, invalidate_canvas_paint};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn clear_expired_toast<H: UiHost, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl CanvasPaintInvalidationCx<H>,
    token: TimerToken,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
{
    if !canvas
        .interaction
        .toast
        .as_ref()
        .is_some_and(|toast| toast.timer == token)
    {
        return false;
    }

    canvas.interaction.toast = None;
    invalidate_canvas_paint(cx);
    true
}
