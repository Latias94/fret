use fret_core::TimerToken;
use fret_ui::UiHost;

use super::widget_tail::{WidgetPaintInvalidationCx, invalidate_widget_paint};
use super::{NodeGraphCanvasMiddleware, NodeGraphCanvasWith};

pub(super) fn clear_expired_toast<H: UiHost, M>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut impl WidgetPaintInvalidationCx<H>,
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
    invalidate_widget_paint(cx);
    true
}
