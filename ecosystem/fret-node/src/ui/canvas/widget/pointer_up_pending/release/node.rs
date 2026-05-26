use fret_ui::UiHost;

use crate::ui::canvas::widget::*;

pub(in super::super) fn handle_pending_node_resize_release<H: UiHost, M, Cx>(
    canvas: &mut NodeGraphCanvasWith<M>,
    cx: &mut Cx,
) -> bool
where
    M: NodeGraphCanvasMiddleware,
    Cx: low_level_adapter::CanvasPointerCaptureReleaseCx<H>,
{
    super::super::super::pointer_up_session::finish_pending_release(
        &mut canvas.interaction.pending_node_resize,
        cx,
    )
}
